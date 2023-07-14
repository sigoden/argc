use crate::command::Command;
use crate::matcher::Matcher;
use crate::utils::{escape_shell_words, get_current_dir, is_windows_path, run_param_fns};
use crate::Result;

use anyhow::bail;
use dirs::home_dir;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::str::FromStr;

pub fn compgen(
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
        if last_arg.starts_with(is_quote) && last_arg.chars().skip(1).all(|v| !is_quote(v)) {
            (last_arg[1..].to_string(), last_arg.chars().next())
        } else {
            (last_arg.to_string(), None)
        }
    };
    let new_args: Vec<String> = args
        .iter()
        .enumerate()
        .map(|(i, v)| {
            if i == args.len() - 1 {
                last.clone()
            } else {
                v.to_string()
            }
        })
        .collect();
    let cmd = Command::new(script_content)?;
    let matcher = Matcher::new(&cmd, &new_args);
    let compgen_values = matcher.compgen(shell);
    let mut default_nospace = unbalance.is_some();
    let mut prefix = unbalance.map(|v| v.to_string()).unwrap_or_default();
    let mut candidates: IndexMap<String, (String, bool, CompKind)> = IndexMap::new();
    let mut argc_fn = None;
    let mut argc_value = None;
    let mut argc_variables = vec![];
    let mut multi_values = HashSet::new();
    if matcher.is_last_arg_option_assign() {
        if let Some((left, right)) = split_equal_sign(&last) {
            prefix.push_str(left);
            last = right.to_string();
            mod_quote(&mut last, &mut prefix, &mut default_nospace);
        }
    }
    for (value, description, comp_kind) in compgen_values {
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
            candidates.insert(value.clone(), (description, default_nospace, comp_kind));
        }
    }
    let mut argc_prefix = prefix.to_string();
    let mut argc_filter = last.to_string();
    let mut argc_cd = None;
    if let Some(fn_name) = argc_fn {
        let output = if script_path.is_empty() {
            let mut values = vec![];
            // complete for argc
            if fn_name == "_choice_completion" {
                if new_args.len() == 3 {
                    values.extend(Shell::list().map(|v| v.name().to_string()))
                }
            } else if fn_name == "_choice_compgen" {
                if new_args.len() == 3 {
                    values.extend(Shell::list().map(|v| v.name().to_string()))
                } else {
                    values.push("__argc_value=file".to_string());
                }
            }
            Some(values.join("\n"))
        } else {
            let mut envs = HashMap::new();
            envs.insert("ARGC_COMPGEN".into(), "1".into());
            envs.insert("ARGC_OS".into(), env::consts::OS.to_string());
            envs.insert("ARGC_PATH_SEP".into(), MAIN_SEPARATOR.into());
            envs.insert("ARGC_DESCRIBE".into(), shell.with_description().to_string());
            envs.insert("ARGC_FILTER".into(), argc_filter.clone());
            envs.insert("ARGC_LAST_ARG".into(), last_arg.to_string());
            if let Some(cwd) = get_current_dir() {
                envs.insert("ARGC_PWD".into(), escape_shell_words(&cwd));
            }
            run_param_fns(script_path, &[fn_name.as_str()], &new_args, envs)
                .map(|output| output[0].clone())
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
                    } else if let Some(stripped_value) = value.strip_prefix("__argc_filter=") {
                        argc_filter = stripped_value.to_string();
                        mod_quote(&mut argc_filter, &mut argc_prefix, &mut default_nospace);
                    } else if let Some(stripped_value) = value.strip_prefix("__argc_cd=") {
                        argc_cd = Some(stripped_value.to_string());
                    }
                    if shell.is_generic() {
                        argc_variables.push(value.to_string());
                    }
                } else if value.starts_with(&argc_filter) && !multi_values.contains(&value) {
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

    if !argc_variables.is_empty() {
        let mut prepend_candicates: IndexMap<String, (String, bool, CompKind)> = argc_variables
            .into_iter()
            .map(|value| (value, (String::new(), false, CompKind::Value)))
            .collect();
        prepend_candicates.extend(candidates);
        candidates = prepend_candicates;
    }

    let mut candidates: Vec<CandidateValue> = candidates
        .into_iter()
        .map(|(value, (description, nospace, comp_kind))| (value, description, nospace, comp_kind))
        .collect();

    if !shell.is_generic() {
        if let Some(argc_value) = argc_value.and_then(|v| convert_arg_value(&v)) {
            if let Some((value_prefix, value_filter, more_candidates)) =
                shell.comp_argc_value(&argc_value, &argc_filter, &argc_cd, default_nospace)
            {
                argc_prefix = format!("{argc_prefix}{value_prefix}");
                argc_filter = value_filter;
                candidates.extend(more_candidates)
            }
        }
    }

    let break_chars = shell.need_break_chars();
    if !break_chars.is_empty() {
        let prefix_quote = unbalance_quote(&argc_prefix);
        if let Some((i, _)) = (match prefix_quote {
            Some((_, idx)) => &argc_prefix[0..idx],
            None => &argc_prefix,
        })
        .char_indices()
        .rfind(|(_, c)| break_chars.contains(c))
        {
            argc_prefix = argc_prefix[i + 1..].to_string();
        }
        if last == argc_filter && prefix_quote.is_none() {
            if let Some((i, _)) = argc_filter
                .char_indices()
                .rfind(|(_, c)| break_chars.contains(c))
            {
                argc_prefix = String::new();
                let idx = i + 1;
                argc_filter = argc_filter[idx..].to_string();
                for (value, _, _, _) in candidates.iter_mut() {
                    *value = value[idx..].to_string()
                }
            };
        }
    }

    let values = shell.convert_candidates(candidates, &argc_prefix, &argc_filter, no_color);

    Ok(values.join("\n"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompKind {
    Flag,
    Option,
    Command,
    Dir,
    File,
    FileExe,
    Symlink,
    ValueOther,
    ValueAnother,
    ValueEmphasis,
    ValueSubtle,
    Value,
}

impl FromStr for CompKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "flag" => Ok(Self::Flag),
            "option" => Ok(Self::Option),
            "command" => Ok(Self::Command),
            "dir" => Ok(Self::Dir),
            "file" => Ok(Self::File),
            "fileexe" => Ok(Self::FileExe),
            "symlink" => Ok(Self::Symlink),
            "valueother" => Ok(Self::ValueOther),
            "valueanother" => Ok(Self::ValueAnother),
            "valueemphasis" => Ok(Self::ValueEmphasis),
            "valuesubtle" => Ok(Self::ValueSubtle),
            _ => Ok(Self::Value),
        }
    }
}

impl CompKind {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Flag => "flag",
            Self::Option => "option",
            Self::Command => "command",
            Self::Dir => "dir",
            Self::File => "file",
            Self::FileExe => "exe",
            Self::Symlink => "symlink",
            Self::ValueOther => "valueOther",
            Self::ValueAnother => "valueAnother",
            Self::ValueEmphasis => "valueEmphasis",
            Self::ValueSubtle => "valueSubtle",
            Self::Value => "value",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    Generic,
    Nushell,
    Powershell,
    Xonsh,
    Zsh,
}

impl FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "bash" => Ok(Self::Bash),
            "elvish" => Ok(Self::Elvish),
            "fish" => Ok(Self::Fish),
            "generic" => Ok(Self::Generic),
            "nushell" => Ok(Self::Nushell),
            "powershell" => Ok(Self::Powershell),
            "xonsh" => Ok(Self::Xonsh),
            "zsh" => Ok(Self::Zsh),
            _ => bail!(
                "The provided shell is either invalid or missing, must be one of {}",
                Shell::list_names(),
            ),
        }
    }
}

pub(crate) type CandidateValue = (String, String, bool, CompKind); // value, description, nospace

impl Shell {
    pub fn list() -> [Shell; 7] {
        [
            Shell::Bash,
            Shell::Elvish,
            Shell::Fish,
            Shell::Nushell,
            Shell::Powershell,
            Shell::Xonsh,
            Shell::Zsh,
        ]
    }

    pub fn list_names() -> String {
        Shell::list()
            .iter()
            .map(|v| v.name())
            .collect::<Vec<&str>>()
            .join(",")
    }

    pub fn name(&self) -> &str {
        match self {
            Shell::Bash => "bash",
            Shell::Elvish => "elvish",
            Shell::Fish => "fish",
            Shell::Generic => "generic",
            Shell::Nushell => "nushell",
            Shell::Powershell => "powershell",
            Shell::Xonsh => "xonsh",
            Shell::Zsh => "zsh",
        }
    }

    pub fn is_generic(&self) -> bool {
        *self == Shell::Generic
    }

    pub(crate) fn convert_candidates(
        &self,
        candidates: Vec<CandidateValue>,
        prefix: &str,
        filter: &str,
        no_color: bool,
    ) -> Vec<String> {
        match self {
            Shell::Bash => {
                let prefix_unbalance = unbalance_quote(prefix);
                let values: Vec<&str> = candidates.iter().map(|(v, _, _, _)| v.as_str()).collect();
                let mut add_space_to_first_candidate = false;
                if values.len() == 1 {
                    let space = if candidates[0].2 { "" } else { " " };
                    let mut value = format!("{prefix}{}", candidates[0].0);
                    if prefix_unbalance.is_none() {
                        value = self.escape(&value);
                    }
                    return vec![format!("{value}{space}")];
                } else if let Some(common) = common_prefix(&values) {
                    if common != filter {
                        if common != "--" {
                            let mut value = format!("{prefix}{common}");
                            if prefix_unbalance.is_none() {
                                value = self.escape(&value);
                            }
                            return vec![value];
                        }
                    } else if !prefix.is_empty() && !common.starts_with(prefix) {
                        add_space_to_first_candidate = true;
                    }
                }
                candidates
                    .into_iter()
                    .enumerate()
                    .map(|(i, (value, description, nospace, _comp_kind))| {
                        let mut new_value = value;
                        if i == 0 && add_space_to_first_candidate {
                            new_value = format!(" {}", new_value)
                        };
                        let description = self.comp_description(&description, "(", ")");
                        if description.is_empty() {
                            let space = if nospace { "" } else { " " };
                            format!("{new_value}{space}")
                        } else {
                            format!("{new_value} {description}")
                        }
                    })
                    .collect::<Vec<String>>()
            }
            Shell::Elvish | Shell::Powershell => candidates
                .into_iter()
                .map(|(value, description, nospace, comp_kind)| {
                    let new_value = self.combine_value(prefix, &value);
                    let display = if value.is_empty() { " ".into() } else { value };
                    let description = self.comp_description(&description, "", "");
                    let space: &str = if nospace { "0" } else { "1" };
                    let color = self.color(comp_kind, no_color);
                    format!("{new_value}\t{space}\t{display}\t{description}\t{color}")
                })
                .collect::<Vec<String>>(),
            Shell::Fish => candidates
                .into_iter()
                .map(|(value, description, _nospace, _comp_kind)| {
                    let new_value = self.combine_value(prefix, &value);
                    let description = self.comp_description(&description, "\t", "");
                    format!("{new_value}{description}")
                })
                .collect::<Vec<String>>(),
            Shell::Generic => candidates
                .into_iter()
                .map(|(value, description, nospace, comp_kind)| {
                    let comp_kind = format!("\t/kind:{}", comp_kind.name());
                    let description = self.comp_description(&description, "\t", "");
                    let space: &str = if nospace { "\0" } else { "" };
                    format!("{value}{space}{comp_kind}{description}")
                })
                .collect::<Vec<String>>(),
            Shell::Nushell => candidates
                .into_iter()
                .map(|(value, description, nospace, _)| {
                    let new_value = self.combine_value(prefix, &value);
                    let space = if nospace { "" } else { " " };
                    let description = self.comp_description(&description, "\t", "");
                    format!("{new_value}{space}{description}")
                })
                .collect::<Vec<String>>(),
            Shell::Xonsh => candidates
                .into_iter()
                .map(|(value, description, nospace, _)| {
                    let mut new_value = format!("{prefix}{value}");
                    if unbalance_quote(prefix).is_none() {
                        let escaped_new_value = self.escape(&new_value);
                        if escaped_new_value.ends_with("\\'")
                            && !escaped_new_value.ends_with("\\\\'")
                        {
                            new_value = format!(
                                "{}\\'",
                                &escaped_new_value.as_str()[0..escaped_new_value.len() - 1]
                            );
                        } else {
                            new_value = escaped_new_value
                        }
                    } else if new_value.ends_with('\\') && !new_value.ends_with("\\\\") {
                        new_value.push('\\')
                    }
                    let display = if value.is_empty() { " ".into() } else { value };
                    let description = self.comp_description(&description, "", "");
                    let space: &str = if nospace { "0" } else { "1" };
                    format!("{new_value}\t{space}\t{display}\t{description}")
                })
                .collect::<Vec<String>>(),
            Shell::Zsh => candidates
                .into_iter()
                .map(|(value, description, nospace, comp_kind)| {
                    let prefix = if let Some((_, i)) = unbalance_quote(prefix) {
                        prefix
                            .char_indices()
                            .filter(|(v, _)| *v != i)
                            .map(|(_, v)| v)
                            .collect()
                    } else {
                        prefix.to_string()
                    };
                    let new_value = self.escape(&format!("{prefix}{value}"));
                    let match_value =
                        escape_chars(&value, self.need_escape_chars(), "\\").replace("\\:", ":");
                    let display = value.replace(':', "\\:");
                    let description = self.comp_description(&description, ":", "");
                    let color = self.color(comp_kind, no_color);
                    let space = if nospace { "" } else { " " };
                    format!("{new_value}{space}\t{display}{description}\t{match_value}\t{color}")
                })
                .collect::<Vec<String>>(),
        }
    }

    pub(crate) fn with_description(&self) -> bool {
        if let Ok(v) = std::env::var("ARGC_COMPGEN_DESCRIPTION") {
            if v == "true" || v == "1" {
                return true;
            } else if v == "false" || v == "0" {
                return false;
            }
        }
        true
    }

    pub(crate) fn escape(&self, value: &str) -> String {
        match self {
            Shell::Bash => escape_chars(value, self.need_escape_chars(), "\\"),
            Shell::Nushell | Shell::Powershell | Shell::Xonsh => {
                if contains_chars(value, self.need_escape_chars()) {
                    format!("'{value}'")
                } else {
                    value.into()
                }
            }
            Shell::Zsh => {
                escape_chars(value, self.need_escape_chars(), "\\\\").replace("\\\\:", "\\:")
            }
            _ => value.into(),
        }
    }

    pub(crate) fn color(&self, comp_kind: CompKind, no_color: bool) -> &'static str {
        match self {
            Shell::Elvish => {
                if no_color {
                    return "default";
                }
                match comp_kind {
                    CompKind::Flag => "cyan",
                    CompKind::Option => "cyan bold",
                    CompKind::Command => "magenta",
                    CompKind::Dir => "blue bold",
                    CompKind::File => "default",
                    CompKind::FileExe => "green bold",
                    CompKind::Symlink => "cyan bold",
                    CompKind::ValueOther => "yellow",
                    CompKind::ValueAnother => "red",
                    CompKind::ValueEmphasis => "green bold",
                    CompKind::ValueSubtle => "green dim",
                    CompKind::Value => "green",
                }
            }
            Shell::Powershell | Shell::Zsh => {
                if no_color {
                    return "39";
                }
                match comp_kind {
                    CompKind::Flag => "36",
                    CompKind::Option => "1;36",
                    CompKind::Command => "35",
                    CompKind::Dir => "1;34",
                    CompKind::File => "39",
                    CompKind::FileExe => "1;32",
                    CompKind::Symlink => "1;36",
                    CompKind::ValueOther => "33",
                    CompKind::ValueAnother => "31",
                    CompKind::ValueEmphasis => "1;32",
                    CompKind::ValueSubtle => "2;32",
                    CompKind::Value => "32",
                }
            }
            _ => "",
        }
    }

    pub(crate) fn redirect_symbols(&self) -> Vec<&str> {
        match self {
            Shell::Nushell => vec![],
            _ => vec!["<", ">"],
        }
    }

    fn is_unix_only(&self) -> bool {
        matches!(self, Shell::Bash | Shell::Fish | Shell::Zsh)
    }

    fn is_windows_mode(&self) -> bool {
        if cfg!(windows) {
            !self.is_unix_only()
        } else {
            false
        }
    }

    fn need_escape_chars(&self) -> &str {
        match self {
            Shell::Bash => r###"()<>"'` !#$&;\|"###,
            Shell::Nushell => r###"()[]{}"'` #$;|"###,
            Shell::Powershell => r###"()<>[]{}"'` #$&,;@|"###,
            Shell::Xonsh => r###"()<>[]{}!"'` #&;|"###,
            Shell::Zsh => r###"()<>[]"'` !#$&*:;?\|"###,
            _ => "",
        }
    }

    fn need_break_chars(&self) -> Vec<char> {
        match self {
            Shell::Bash => match std::env::var("COMP_WORDBREAKS") {
                Ok(v) => v.chars().collect(),
                Err(_) => vec!['=', ':', '|', ';', '\'', '"'],
            },
            Shell::Powershell => vec![','],
            _ => vec![],
        }
    }

    fn need_expand_tilde(&self) -> bool {
        self.is_windows_mode() && self == &Self::Powershell
    }

    fn comp_argc_value(
        &self,
        argc_value: &str,
        value: &str,
        cd: &Option<String>,
        default_nospace: bool,
    ) -> Option<(String, String, Vec<CandidateValue>)> {
        let (dir_only, exts) = if argc_value == "__argc_value=dir" {
            (true, None)
        } else if argc_value == "__argc_value=file" {
            (false, None)
        } else if let Some(stripped_value) = argc_value.strip_prefix("__argc_value=file:") {
            let exts: Vec<String> = stripped_value.split(',').map(|v| v.to_string()).collect();
            (false, Some(exts))
        } else {
            return None;
        };
        let (search_dir, filter, prefix) = self.resolve_path(value, cd)?;
        if !search_dir.is_dir() {
            return None;
        }

        let is_windows_mode = self.is_windows_mode();
        let exts = exts.unwrap_or_default();

        let (value_prefix, path_sep) = if is_windows_mode {
            if !value.contains(&prefix) {
                return Some((
                    "".into(),
                    "".into(),
                    vec![(format!("{prefix}{filter}"), "".into(), true, CompKind::Dir)],
                ));
            }
            let value_prefix = if prefix.is_empty() { ".\\" } else { "" };
            (value_prefix, "\\")
        } else {
            if value == "~" {
                return Some((
                    "".into(),
                    "".into(),
                    vec![("~/".into(), "".into(), true, CompKind::Dir)],
                ));
            }
            ("", "/")
        };

        #[cfg(windows)]
        let path_exts: Vec<String> = env::var("PATHEXT")
            .unwrap_or(".COM;.EXE;.BAT;.CMD".into())
            .split(';')
            .filter(|v| !v.is_empty())
            .map(|v| v.to_lowercase())
            .collect();

        let mut output = vec![];

        for entry in (fs::read_dir(&search_dir).ok()?).flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.starts_with(&filter) {
                continue;
            }
            if (filter.is_empty() || !filter.starts_with('.'))
                && !is_windows_mode
                && file_name.starts_with('.')
            {
                continue;
            };

            let mut meta = match fs::symlink_metadata(&path) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let is_symlink = meta.is_symlink();

            if is_symlink {
                meta = match fs::metadata(&path) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
            }
            let is_dir = meta.is_dir();
            if !is_dir && dir_only {
                continue;
            }
            if !is_dir
                && !exts.is_empty()
                && exts.iter().all(|v| !file_name.to_lowercase().ends_with(v))
            {
                continue;
            }
            let path_value = if is_dir {
                format!("{value_prefix}{file_name}{}", path_sep)
            } else {
                format!("{value_prefix}{file_name}")
            };

            let comp_kind = if is_dir {
                CompKind::Dir
            } else if is_symlink {
                CompKind::Symlink
            } else {
                #[cfg(not(windows))]
                let is_executable = {
                    use std::os::unix::fs::PermissionsExt;
                    meta.permissions().mode() & 0o111 != 0
                };
                #[cfg(windows)]
                let is_executable = {
                    let new_file_name = file_name.to_lowercase();
                    path_exts.iter().any(|v| new_file_name.ends_with(v))
                };
                if is_executable {
                    CompKind::FileExe
                } else {
                    CompKind::File
                }
            };
            let nospace = if default_nospace { true } else { is_dir };
            output.push((path_value, String::new(), nospace, comp_kind))
        }

        output.sort_by(|a, b| a.0.cmp(&b.0));

        Some((prefix, filter, output))
    }

    fn resolve_path(&self, value: &str, cd: &Option<String>) -> Option<(PathBuf, String, String)> {
        let is_windows_mode = self.is_windows_mode();
        let (value, sep, home_p, cwd_p) = if is_windows_mode {
            (value.replace('/', "\\"), "\\", "~\\", ".\\")
        } else {
            (value.to_string(), "/", "~/", "./")
        };
        let (new_value, trims, prefix) = if value.starts_with(home_p) || value == "~" {
            let home = home_dir()?;
            let home_s = home.to_string_lossy();
            let path_s = if value == "~" {
                format!("{home_s}{sep}")
            } else {
                format!("{home_s}{}", &value[1..])
            };
            let (trims, prefix) = if self.need_expand_tilde() {
                (0, "")
            } else {
                (home_s.len(), "~")
            };
            (path_s, trims, prefix.into())
        } else if value.starts_with(sep) {
            let new_value = if is_windows_mode {
                format!("C:{}", value)
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
            let mut cwd = env::current_dir().ok()?;
            if let Some(cd) = cd {
                cwd = cwd.join(cd).canonicalize().ok()?;
            }
            let cwd_s = cwd.to_string_lossy();
            let new_value = format!("{}{sep}{new_value}", cwd_s);
            let trims = cwd_s.len() + 1;
            (new_value, trims, prefix.to_string())
        };
        let (path, filter) = if new_value.ends_with(sep) {
            (new_value, "".into())
        } else if value == "~" {
            (format!("{new_value}{sep}"), "".into())
        } else {
            let (parent, filter) = new_value.rsplit_once(sep)?;
            (format!("{parent}{sep}"), filter.into())
        };
        let prefix = format!("{prefix}{}", &path[trims..]);
        Some((PathBuf::from(path), filter, prefix))
    }

    fn combine_value(&self, prefix: &str, value: &str) -> String {
        let mut new_value = format!("{prefix}{value}");
        if unbalance_quote(prefix).is_none() {
            new_value = self.escape(&new_value);
        }
        new_value
    }

    fn comp_description(&self, description: &str, prefix: &str, suffix: &str) -> String {
        if description.is_empty() || !self.with_description() {
            String::new()
        } else {
            format!("{prefix}{}{suffix}", truncate_description(description))
        }
    }
}

fn parse_candidate_value(input: &str) -> CandidateValue {
    let parts: Vec<&str> = input.split('\t').collect();
    let parts_len = parts.len();
    let mut value = String::new();
    let mut description = String::new();
    let mut nospace = false;
    let mut comp_kind = CompKind::Value;
    if parts_len >= 2 {
        if let Some(kind_value) = parts[1].strip_prefix("/kind:").and_then(|v| v.parse().ok()) {
            comp_kind = kind_value;
            description = parts[2..].join("\t");
        } else {
            description = parts[1..].join("\t");
        };
    }
    if parts_len > 0 {
        if let Some(stripped_value) = parts.first().and_then(|v| v.strip_suffix('\0')) {
            value = stripped_value.to_string();
            nospace = true;
        } else {
            value = parts[0].to_string();
        }
    }
    (value, description, nospace, comp_kind)
}

fn convert_arg_value(value: &str) -> Option<String> {
    if value.starts_with("file:") {
        Some(format!("__argc_value={value}"))
    } else if ["path", "file", "arg", "any"]
        .iter()
        .any(|v| value.contains(v))
    {
        Some("__argc_value=file".to_string())
    } else if value.contains("dir") || value.contains("folder") {
        Some("__argc_value=dir".to_string())
    } else {
        None
    }
}

fn truncate_description(description: &str) -> String {
    let max_width = 80;
    let description = description.trim().replace('\t', "");
    if description.len() < max_width {
        description
    } else {
        format!("{}...", &description[0..max_width - 3])
    }
}

fn unbalance_quote(value: &str) -> Option<(char, usize)> {
    if value.starts_with(is_quote) {
        let ch = value.chars().next()?;
        if value.chars().filter(|c| *c == ch).count() % 2 == 1 {
            return Some((ch, 0));
        }
    } else if value.ends_with(is_quote) {
        let ch = value.chars().rev().next()?;
        if value.chars().filter(|c| *c == ch).count() % 2 == 1 {
            return Some((ch, value.len() - 1));
        }
    }
    None
}

fn split_equal_sign(word: &str) -> Option<(&str, &str)> {
    let chars: Vec<char> = word
        .chars()
        .skip_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if let Some('=') = chars.first() {
        let idx = word.len() - chars.len() + 1;
        if idx == 1 {
            return None;
        }
        return Some((&word[..idx], &word[idx..]));
    }
    None
}

fn common_prefix(strings: &[&str]) -> Option<String> {
    if strings.is_empty() {
        return None;
    }
    let mut prefix = String::new();
    for (i, c) in strings[0].chars().enumerate() {
        for s in &strings[1..] {
            if i >= s.len() || s.chars().nth(i) != Some(c) {
                if prefix.is_empty() {
                    return None;
                }
                return Some(prefix);
            }
        }
        prefix.push(c);
    }
    if prefix.is_empty() {
        return None;
    }
    Some(prefix)
}

fn escape_chars(value: &str, need_escape: &str, for_escape: &str) -> String {
    let chars: Vec<char> = need_escape.chars().collect();
    value
        .chars()
        .map(|c| {
            if chars.contains(&c) {
                format!("{for_escape}{c}")
            } else {
                c.to_string()
            }
        })
        .collect()
}

fn mod_quote(filter: &mut String, prefix: &mut String, default_nospace: &mut bool) {
    if filter.starts_with(is_quote) {
        prefix.push_str(&filter[0..1]);
        *default_nospace = true;
    }
    *filter = filter.trim_matches(is_quote).to_string();
}

fn contains_chars(value: &str, chars: &str) -> bool {
    let value_chars: Vec<char> = value.chars().collect();
    chars.chars().any(|v| value_chars.contains(&v))
}

fn is_quote(c: char) -> bool {
    c == '\'' || c == '"'
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_parse_candidate_value {
        ($input:expr, $value:expr, $desc:expr, $nospace:expr, $comp_kind:expr) => {
            assert_eq!(
                parse_candidate_value($input),
                ($value.to_string(), $desc.to_string(), $nospace, $comp_kind)
            )
        };
    }

    #[test]
    fn test_parse_candidate_value() {
        assert_parse_candidate_value!("abc", "abc", "", false, CompKind::Value);
        assert_parse_candidate_value!("abc\0", "abc", "", true, CompKind::Value);
        assert_parse_candidate_value!("abc\tA value", "abc", "A value", false, CompKind::Value);
        assert_parse_candidate_value!("abc\0\tA value", "abc", "A value", true, CompKind::Value);
        assert_parse_candidate_value!(
            "abc\0\tA value\tmore desc",
            "abc",
            "A value\tmore desc",
            true,
            CompKind::Value
        );
        assert_parse_candidate_value!(
            "abc\0\t/kind:command\tA value\tmore desc",
            "abc",
            "A value\tmore desc",
            true,
            CompKind::Command
        );
        assert_parse_candidate_value!(
            "abc\0\t/kind:command\tA value",
            "abc",
            "A value",
            true,
            CompKind::Command
        );
        assert_parse_candidate_value!("abc\0\t/kind:command", "abc", "", true, CompKind::Command);
        assert_parse_candidate_value!("abc\t/kind:command", "abc", "", false, CompKind::Command);
        assert_parse_candidate_value!("", "", "", false, CompKind::Value);
    }

    #[test]
    fn test_split_equal_sign() {
        assert_eq!(split_equal_sign("-a="), Some(("-a=", "")));
        assert_eq!(split_equal_sign("a="), Some(("a=", "")));
        assert_eq!(split_equal_sign("-a=c"), Some(("-a=", "c")));
        assert_eq!(split_equal_sign("a"), None);
        assert_eq!(split_equal_sign("a:"), None);
        assert_eq!(split_equal_sign("=a"), None);
    }
}
