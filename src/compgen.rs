use crate::command::Command;
use crate::matcher::Matcher;
use crate::runtime::Runtime;
use crate::utils::{is_quote_char, is_windows_path, unbalance_quote};
use crate::Shell;

use anyhow::{bail, Result};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

pub const COMPGEN_KIND_SYMBOL: &str = "___compgen_kind___";

const FILE_PROTO: &str = "file://";

pub fn compgen<T: Runtime>(
    runtime: T,
    shell: Shell,
    script_path: &str,
    script_content: &str,
    args: &[String],
    no_color: bool,
) -> Result<String> {
    if args.len() < 2 {
        return Ok(String::new());
    }
    let last_arg = args[args.len() - 1].as_str();
    let (mut last, unbalance) = {
        if last_arg.starts_with(is_quote_char)
            && last_arg.chars().skip(1).all(|v| !is_quote_char(v))
        {
            (last_arg[1..].to_string(), last_arg.chars().next())
        } else {
            (last_arg.to_string(), None)
        }
    };
    let cmd = if script_path == COMPGEN_KIND_SYMBOL {
        let comp_kind = &args[0];
        let script_content = format!(
            r#"# @arg args~[`{comp_kind}`]
{comp_kind} () {{ :; }}
"#
        );
        Command::new(&script_content, &args[0])?
    } else {
        Command::new(script_content, &args[0])?
    };
    let new_args: Vec<String> = if cmd.delegated() {
        args.to_vec()
    } else {
        args.iter()
            .enumerate()
            .map(|(i, v)| {
                if i == args.len() - 1 {
                    last.clone()
                } else {
                    v.to_string()
                }
            })
            .collect()
    };
    let matcher = Matcher::new(runtime, &cmd, &new_args, true);
    let compgen_values = matcher.compgen(shell);
    let mut default_nospace = unbalance.is_some();
    let mut prefix = unbalance.map(|v| v.to_string()).unwrap_or_default();
    let mut candidates: IndexMap<String, (String, bool, CompColor)> = IndexMap::new();
    let mut argc_fn = None;
    let mut argc_value = None;
    let mut argc_variables = vec![];
    let mut multi_values = HashSet::new();
    if let Some(at) = matcher.split_last_arg_at() {
        let (left, right) = last.split_at(at);
        prefix.push_str(left);
        last = right.to_string();
        mod_quote(&mut last, &mut prefix, &mut default_nospace);
    }
    for (value, description, nospace, comp_color) in compgen_values {
        if value.starts_with("__argc_") {
            if let Some(fn_name) = value.strip_prefix("__argc_fn=") {
                argc_fn = Some(fn_name.to_string());
            } else if let Some(stripped_value) = value.strip_prefix("__argc_value=") {
                argc_value = Some(stripped_value.to_lowercase());
                if shell.is_generic() {
                    argc_variables.push(value.to_string());
                }
            } else if let Some(value) = value.strip_prefix("__argc_multi=") {
                if let Some(ch) = value.chars().next() {
                    default_nospace = true;
                    if let Some((i, _)) = last.char_indices().rfind(|(_, c)| ch == *c) {
                        multi_values = last[..i].split(ch).map(|v| v.to_string()).collect();
                        let idx = i + 1;
                        prefix.push_str(&last[..idx]);
                        last = last[idx..].to_string();
                        mod_quote(&mut last, &mut prefix, &mut default_nospace);
                    }
                }
            }
        } else if value.starts_with(&last) && !multi_values.contains(&value) {
            candidates.insert(
                value.clone(),
                (description, nospace || default_nospace, comp_color),
            );
        }
    }
    let mut argc_prefix = prefix.to_string();
    let mut argc_filter = last.to_string();
    let mut argc_suffix = String::new();
    let mut argc_cd = None;
    if let Some(func) = argc_fn {
        let output = if script_path == COMPGEN_KIND_SYMBOL {
            let mut values = vec![];
            let comp_kind = CompKind::new(&func);
            match comp_kind {
                CompKind::Shell => values.extend(Shell::list().map(|v| v.name().to_string())),
                CompKind::Path => values.push("__argc_value=path".to_string()),
                CompKind::Dir => values.push("__argc_value=dir".to_string()),
                CompKind::Unknown => {}
            }
            Some(values.join("\n"))
        } else if !script_path.is_empty() {
            let mut envs = HashMap::new();
            envs.insert("ARGC_COMPGEN".into(), "1".into());
            envs.insert("ARGC_OS".into(), runtime.os());
            envs.insert("ARGC_CWORD".into(), argc_filter.clone());
            envs.insert("ARGC_LAST_ARG".into(), last_arg.to_string());
            runtime
                .exec_bash_functions(script_path, &[func.as_str()], &new_args, envs)
                .and_then(|output| output.first().cloned())
        } else {
            None
        };
        if let Some(output) = output.and_then(|v| if v.is_empty() { None } else { Some(v) }) {
            for line in output.trim().split('\n').map(|v| v.trim()) {
                let (value, description, nospace, comp_type) = parse_candidate_value(line);
                let nospace = nospace || default_nospace;
                if value.starts_with("__argc_") {
                    if let Some(stripped_value) = value.strip_prefix("__argc_value=") {
                        argc_value = Some(stripped_value.to_lowercase());
                    } else if let Some(stripped_value) = value.strip_prefix("__argc_prefix=") {
                        argc_prefix = format!("{prefix}{stripped_value}");
                    } else if let Some(stripped_value) = value.strip_prefix("__argc_suffix=") {
                        if nospace {
                            default_nospace = true;
                        }
                        argc_suffix = stripped_value.to_string();
                    } else if let Some(stripped_value) = value.strip_prefix("__argc_filter=") {
                        argc_filter = stripped_value.to_string();
                        mod_quote(&mut argc_filter, &mut argc_prefix, &mut default_nospace);
                    } else if let Some(stripped_value) = value.strip_prefix("__argc_cd=") {
                        argc_cd = Some(stripped_value.to_string());
                    }
                    if shell.is_generic() {
                        argc_variables.push(value.to_string());
                    }
                } else {
                    let value = format!("{value}{argc_suffix}");
                    if value.starts_with(&argc_filter) && !multi_values.contains(&value) {
                        match candidates.get_mut(&value) {
                            Some((v1, v2, _)) => {
                                if v1.is_empty() && !description.is_empty() {
                                    *v1 = description.to_string();
                                }
                                if !*v2 && nospace {
                                    *v2 = true
                                }
                            }
                            None => {
                                candidates.insert(
                                    value.to_string(),
                                    (description.to_string(), nospace, comp_type),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    if !argc_variables.is_empty() {
        let mut prepend_candidates: IndexMap<String, (String, bool, CompColor)> = argc_variables
            .into_iter()
            .map(|value| (value, (String::new(), false, CompColor::of_value())))
            .collect();
        prepend_candidates.extend(candidates);
        candidates = prepend_candidates;
    }

    let need_description = shell.need_description(runtime);

    let mut candidates: Vec<CandidateValue> = candidates
        .into_iter()
        .map(|(value, (description, nospace, comp_color))| {
            let description = if need_description {
                description
            } else {
                String::new()
            };
            (value, description, nospace, comp_color)
        })
        .collect();

    if !shell.is_generic() {
        if let Some(path_value) = argc_value.and_then(|v| convert_arg_value(&v)) {
            if let Some((value_prefix, value_filter, more_candidates)) = path_value.compgen(
                runtime,
                shell,
                &argc_filter,
                &argc_suffix,
                &argc_cd,
                default_nospace,
            ) {
                if candidates.is_empty() || value_prefix.is_empty() {
                    argc_prefix = format!("{argc_prefix}{value_prefix}");
                    argc_filter = value_filter;
                    candidates.extend(more_candidates)
                } else {
                    candidates.extend(more_candidates.into_iter().map(|mut v| {
                        v.0 = format!("{value_prefix}{}", v.0);
                        v
                    }))
                }
            }
        }
    }

    let break_chars = shell.need_break_chars(runtime, &argc_prefix);
    if !break_chars.is_empty() {
        let prefix = format!("{argc_prefix}{argc_filter}");
        let prefix = match unbalance_quote(&prefix) {
            Some((_, i)) => prefix.chars().take(i.saturating_sub(1)).collect(),
            None => prefix,
        };
        if let Some((break_at, break_char)) = prefix
            .char_indices()
            .rfind(|(_, c)| break_chars.contains(c))
        {
            let argc_prefix_len = argc_prefix.len();
            let offset = if shell == Shell::Bash && break_char == '@' {
                // I don't know why `@` (as a break char) is weird in bash
                0
            } else {
                1
            };

            if break_at < argc_prefix_len {
                argc_prefix = argc_prefix[break_at + offset..].to_string();
            } else {
                argc_prefix = String::new();
                let idx = break_at - argc_prefix_len + offset;
                argc_filter = argc_filter[idx..].to_string();
                for (value, _, _, _) in candidates.iter_mut() {
                    *value = value[idx..].to_string()
                }
            }
        }
    }

    let values = shell.convert_candidates(candidates, &argc_prefix, &argc_filter, no_color);

    Ok(values.join("\n"))
}

pub fn compgen_kind<T: Runtime>(
    runtime: T,
    shell: Shell,
    kind: CompKind,
    last_arg: &str,
    no_color: bool,
) -> Result<String> {
    let args = [kind.name().to_string(), last_arg.to_string()];
    compgen(runtime, shell, COMPGEN_KIND_SYMBOL, "", &args, no_color)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompKind {
    Shell,
    Path,
    Dir,
    Unknown,
}

impl CompKind {
    pub fn new(value: &str) -> Self {
        match value {
            "shell" => CompKind::Shell,
            "path" => CompKind::Path,
            "dir" => CompKind::Dir,
            _ => CompKind::Unknown,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            CompKind::Shell => "shell",
            CompKind::Path => "path",
            CompKind::Dir => "dir",
            CompKind::Unknown => "unknown",
        }
    }
}

pub(crate) type CandidateValue = (String, String, bool, CompColor); // value, description, nospace, comp_color

/// (value, description, nospace, color)
pub(crate) type CompItem = (String, String, bool, CompColor);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CompColor {
    pub(crate) code: ColorCode,
    pub(crate) style: ColorStyle,
}

impl CompColor {
    pub(crate) fn new(code: ColorCode, style: ColorStyle) -> Self {
        Self { code, style }
    }

    pub(crate) fn parse(s: &str) -> Result<Self> {
        if let Some((code, style)) = s.split_once(',') {
            if let (Ok(code), Ok(style)) = (code.parse(), style.parse()) {
                return Ok(Self::new(code, style));
            }
        } else if let Ok(code) = s.parse() {
            return Ok(Self::new(code, ColorStyle::Regular));
        }
        bail!("Invalid CompColor value")
    }

    pub(crate) fn ser(&self) -> String {
        if self.style == ColorStyle::Regular {
            format!("{}", self.code)
        } else {
            format!("{},{}", self.code, self.style)
        }
    }

    pub(crate) fn ansi_code(&self) -> String {
        let mut ret = if self.style != ColorStyle::Regular {
            format!("{};", self.style.ansi_code())
        } else {
            String::new()
        };
        ret.push_str(self.code.ansi_code());
        ret
    }

    pub(crate) fn style(&self) -> String {
        let mut ret = self.code.to_string();
        if self.style != ColorStyle::Regular {
            ret.push_str(&format!(" {}", self.style))
        }
        ret
    }

    pub(crate) fn of_flag() -> Self {
        Self::new(ColorCode::Cyan, ColorStyle::Regular)
    }

    pub(crate) fn of_option() -> Self {
        Self::new(ColorCode::Cyan, ColorStyle::Bold)
    }

    pub(crate) fn of_command() -> Self {
        Self::new(ColorCode::Magenta, ColorStyle::Regular)
    }

    pub(crate) fn of_dir() -> Self {
        Self::new(ColorCode::Blue, ColorStyle::Bold)
    }

    pub(crate) fn of_file() -> Self {
        Self::new(ColorCode::Default, ColorStyle::Regular)
    }

    pub(crate) fn of_file_exe() -> Self {
        Self::new(ColorCode::Green, ColorStyle::Bold)
    }

    pub(crate) fn of_symlink() -> Self {
        Self::new(ColorCode::Cyan, ColorStyle::Bold)
    }

    pub(crate) fn of_value() -> Self {
        Self::new(ColorCode::Default, ColorStyle::Regular)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorCode {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
}

impl FromStr for ColorCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "black" => Ok(Self::Black),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "yellow" => Ok(Self::Yellow),
            "blue" => Ok(Self::Blue),
            "magenta" => Ok(Self::Magenta),
            "cyan" => Ok(Self::Cyan),
            "white" => Ok(Self::White),
            "default" => Ok(Self::Default),
            _ => bail!("Invalid ColorCode value"),
        }
    }
}

impl fmt::Display for ColorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Black => write!(f, "black"),
            Self::Red => write!(f, "red"),
            Self::Green => write!(f, "green"),
            Self::Yellow => write!(f, "yellow"),
            Self::Blue => write!(f, "blue"),
            Self::Magenta => write!(f, "magenta"),
            Self::Cyan => write!(f, "cyan"),
            Self::White => write!(f, "white"),
            Self::Default => write!(f, "default"),
        }
    }
}

impl ColorCode {
    pub fn ansi_code(&self) -> &str {
        match self {
            Self::Black => "30",
            Self::Red => "31",
            Self::Green => "32",
            Self::Yellow => "33",
            Self::Blue => "34",
            Self::Magenta => "35",
            Self::Cyan => "36",
            Self::White => "37",
            Self::Default => "39",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorStyle {
    Regular,
    Bold,
}

impl FromStr for ColorStyle {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "regular" => Ok(Self::Regular),
            "bold" => Ok(Self::Bold),
            _ => bail!("Invalid CodeStyle value"),
        }
    }
}

impl fmt::Display for ColorStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Regular => write!(f, "regular"),
            Self::Bold => write!(f, "bold"),
        }
    }
}

impl ColorStyle {
    pub fn ansi_code(&self) -> &str {
        match self {
            Self::Regular => "0",
            Self::Bold => "1",
        }
    }
}

pub(crate) struct ArgcPathValue {
    pub(crate) is_dir: bool,
    pub(crate) exts: Vec<String>,
}

impl ArgcPathValue {
    fn compgen<T: Runtime>(
        &self,
        runtime: T,
        shell: Shell,
        value: &str,
        suffix: &str,
        cd: &Option<String>,
        default_nospace: bool,
    ) -> Option<(String, String, Vec<CandidateValue>)> {
        let is_windows_mode = shell.is_windows_mode(runtime);
        let need_expand_tilde = shell.need_expand_tilde(runtime);
        let is_file_proto = value.starts_with(FILE_PROTO);
        let (search_dir, filter, prefix) =
            self.resolve_path(runtime, value, cd, is_windows_mode, need_expand_tilde)?;
        let (is_dir, _, _) = runtime.metadata(&search_dir)?;
        if !is_dir {
            return None;
        }

        let (value_prefix, path_sep) = if is_windows_mode {
            if !value.contains(&prefix) {
                return Some((
                    "".into(),
                    "".into(),
                    vec![(
                        format!("{prefix}{filter}"),
                        "".into(),
                        true,
                        CompColor::of_value(),
                    )],
                ));
            }
            let value_prefix = if prefix.is_empty() { ".\\" } else { "" };
            let path_sep = if is_file_proto { "/" } else { "\\" };
            (value_prefix, path_sep)
        } else {
            if value == "~" {
                return Some((
                    "".into(),
                    "".into(),
                    vec![("~/".into(), "".into(), true, CompColor::of_dir())],
                ));
            }
            ("", "/")
        };

        let mut output = vec![];

        for file_name in runtime.read_dir(&search_dir)? {
            let path = runtime.join_path(&search_dir, &[&file_name]);
            if !file_name.starts_with(&filter) {
                continue;
            }
            if (filter.is_empty() || !filter.starts_with('.'))
                && !is_windows_mode
                && file_name.starts_with('.')
            {
                continue;
            };

            let (is_dir, is_symlink, is_executable) = match runtime.metadata(&path) {
                Some(v) => v,
                _ => continue,
            };
            if !is_dir && self.is_dir {
                continue;
            }
            if !is_dir
                && !self.exts.is_empty()
                && self
                    .exts
                    .iter()
                    .all(|v| !file_name.to_lowercase().ends_with(v))
            {
                continue;
            }
            let path_value = if is_dir {
                format!("{value_prefix}{file_name}{path_sep}")
            } else {
                format!("{value_prefix}{file_name}{suffix}")
            };

            let comp_color = if is_dir {
                CompColor::of_dir()
            } else if is_symlink {
                CompColor::of_symlink()
            } else if is_executable {
                CompColor::of_file_exe()
            } else {
                CompColor::of_file()
            };
            let nospace = if default_nospace { true } else { is_dir };
            output.push((path_value, String::new(), nospace, comp_color))
        }

        output.sort_by(|a, b| natord::compare_ignore_case(&a.0, &b.0));

        Some((prefix, filter, output))
    }

    fn resolve_path<T: Runtime>(
        &self,
        runtime: T,
        value: &str,
        cd: &Option<String>,
        is_windows_mode: bool,
        need_expand_tilde: bool,
    ) -> Option<(String, String, String)> {
        let is_home = value == "~";
        let is_file_proto = value.starts_with(FILE_PROTO);
        let (value, sep, home_p, cwd_p) = if is_windows_mode && !is_file_proto {
            (value.replace('/', "\\"), "\\", "~\\", ".\\")
        } else {
            (value.to_string(), "/", "~/", "./")
        };
        let (new_value, trims, prefix) = if let Some(value) = value.strip_prefix(FILE_PROTO) {
            let value = value.replace('\\', "/");
            if value.is_empty() {
                if is_windows_mode {
                    ("C:/".into(), 0, format!("{FILE_PROTO}/"))
                } else {
                    ("/".into(), 0, FILE_PROTO.into())
                }
            } else if let Some(trimmed_value) = value.strip_prefix('/') {
                if is_windows_mode {
                    let new_value = if trimmed_value.is_empty() {
                        "C:/".into()
                    } else if is_windows_path(trimmed_value) {
                        if trimmed_value.len() == 2 {
                            format!("{trimmed_value}/")
                        } else {
                            trimmed_value.into()
                        }
                    } else {
                        trimmed_value.into()
                    };
                    (new_value, 0, format!("{FILE_PROTO}/"))
                } else {
                    (value.to_string(), 0, FILE_PROTO.into())
                }
            } else {
                let mut cwd = runtime.current_dir()?;
                if let Some(cd) = cd {
                    cwd = runtime.chdir(&cwd, cd)?;
                }
                let trims = cwd.len() + 1;
                let new_value = format!("{cwd}{sep}{value}").replace('\\', "/");
                (new_value, trims, FILE_PROTO.into())
            }
        } else if is_home || value.starts_with(home_p) {
            let home = dirs::home_dir()?;
            let home_s = home.to_string_lossy();
            let path_s = if is_home {
                format!("{home_s}{sep}")
            } else {
                format!("{home_s}{}", &value[1..])
            };
            let (trims, prefix) = if need_expand_tilde {
                (0, "")
            } else {
                (home_s.len(), "~")
            };
            (path_s, trims, prefix.into())
        } else if value.starts_with(sep) {
            let new_value = if is_windows_mode {
                format!("C:{value}")
            } else {
                value.clone()
            };
            (new_value, 0, "".into())
        } else if is_windows_mode && is_windows_path(&value) {
            let new_value = if value.len() == 2 {
                format!("{value}{sep}")
            } else {
                value.clone()
            };
            (new_value, 0, "".into())
        } else {
            let (new_value, prefix) = if let Some(value) = value.strip_prefix(cwd_p) {
                (value.to_string(), cwd_p)
            } else {
                (value.clone(), "")
            };
            let mut cwd = runtime.current_dir()?;
            if let Some(cd) = cd {
                cwd = runtime.chdir(&cwd, cd)?;
            }
            let new_value = format!("{cwd}{sep}{new_value}");
            let trims = cwd.len() + 1;
            (new_value, trims, prefix.to_string())
        };
        let (path, filter) = if new_value.ends_with(sep) {
            (new_value, "".into())
        } else if is_home {
            (format!("{new_value}{sep}"), "".into())
        } else {
            let (parent, filter) = new_value.rsplit_once(sep)?;
            (format!("{parent}{sep}"), filter.into())
        };
        let prefix = format!("{prefix}{}", &path[trims..]);
        Some((path, filter, prefix))
    }
}

fn convert_arg_value(value: &str) -> Option<ArgcPathValue> {
    if let Some(stripped_value) = value
        .strip_prefix("path:")
        .or_else(|| value.strip_prefix("file:"))
    {
        Some(ArgcPathValue {
            is_dir: false,
            exts: stripped_value.split(',').map(|v| v.to_string()).collect(),
        })
    } else if ["path", "file", "arg", "any"]
        .iter()
        .any(|v| value.contains(v))
    {
        Some(ArgcPathValue {
            is_dir: false,
            exts: vec![],
        })
    } else if value.contains("dir") || value.contains("folder") {
        Some(ArgcPathValue {
            is_dir: true,
            exts: vec![],
        })
    } else {
        None
    }
}

fn parse_candidate_value(input: &str) -> CandidateValue {
    let parts: Vec<&str> = input.split('\t').collect();
    let parts_len = parts.len();
    let mut value = String::new();
    let mut description = String::new();
    let mut nospace = false;
    let mut comp_color = CompColor::of_value();
    if parts_len >= 2 {
        if let Some(color) = parts[1]
            .strip_prefix("/color:")
            .and_then(|v| CompColor::parse(v).ok())
        {
            comp_color = color;
            description = parts[2..].join("\t");
        } else {
            description = parts[1..].join("\t");
        };
    }
    if parts_len > 0 {
        if let Some(stripped_value) = parts.first().and_then(|v| v.strip_suffix('\0')) {
            value = stripped_value.trim().to_string();
            nospace = true;
        } else {
            value = parts[0].trim().to_string();
        }
    }
    (value, description, nospace, comp_color)
}

fn mod_quote(filter: &mut String, prefix: &mut String, default_nospace: &mut bool) {
    if filter.starts_with(is_quote_char) {
        prefix.push_str(&filter[0..1]);
        *default_nospace = true;
    }
    *filter = filter.trim_matches(is_quote_char).to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_parse_candidate_value {
        ($input:expr, $value:expr, $desc:expr, $nospace:expr, $comp_color:expr) => {
            assert_eq!(
                parse_candidate_value($input),
                ($value.to_string(), $desc.to_string(), $nospace, $comp_color)
            )
        };
    }

    #[test]
    fn test_parse_candidate_value() {
        assert_parse_candidate_value!("abc", "abc", "", false, CompColor::of_value());
        assert_parse_candidate_value!("abc\0", "abc", "", true, CompColor::of_value());
        assert_parse_candidate_value!(
            "abc\tA value",
            "abc",
            "A value",
            false,
            CompColor::of_value()
        );
        assert_parse_candidate_value!(
            "abc\0\tA value",
            "abc",
            "A value",
            true,
            CompColor::of_value()
        );
        assert_parse_candidate_value!(
            "abc\0\tA value\tmore desc",
            "abc",
            "A value\tmore desc",
            true,
            CompColor::of_value()
        );
        assert_parse_candidate_value!(
            "abc\0\t/color:magenta\tA value\tmore desc",
            "abc",
            "A value\tmore desc",
            true,
            CompColor::of_command()
        );
        assert_parse_candidate_value!(
            "abc\0\t/color:cyan,bold\tA value",
            "abc",
            "A value",
            true,
            CompColor::of_option()
        );
        assert_parse_candidate_value!("abc\0\t/color:cyan", "abc", "", true, CompColor::of_flag());
        assert_parse_candidate_value!(
            "abc\t/color:default",
            "abc",
            "",
            false,
            CompColor::of_file()
        );
        assert_parse_candidate_value!("", "", "", false, CompColor::of_value());
    }
}
