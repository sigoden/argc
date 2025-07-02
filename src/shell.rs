use std::str::FromStr;

use anyhow::bail;

use crate::utils::{is_quote_char, is_true_value, unbalance_quote};

#[cfg(feature = "compgen")]
use crate::{
    compgen::{CandidateValue, CompColor},
    Runtime,
};

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
    Tcsh,
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
            "tcsh" => Ok(Self::Tcsh),
            _ => bail!(
                "The provided shell is either invalid or missing, must be one of {}",
                Shell::list_names(),
            ),
        }
    }
}

impl Shell {
    pub fn list() -> [Shell; 8] {
        [
            Shell::Bash,
            Shell::Elvish,
            Shell::Fish,
            Shell::Nushell,
            Shell::Powershell,
            Shell::Xonsh,
            Shell::Zsh,
            Shell::Tcsh,
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
            Shell::Tcsh => "tcsh",
        }
    }

    pub fn is_generic(&self) -> bool {
        *self == Shell::Generic
    }

    pub fn is_unix_only(&self) -> bool {
        matches!(self, Shell::Bash | Shell::Fish | Shell::Zsh | Shell::Tcsh)
    }
}

#[cfg(feature = "compgen")]
impl Shell {
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
                } else if let Some(common) = Self::common_prefix(&values) {
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
                    .map(|(i, (value, description, nospace, _comp_color))| {
                        let mut new_value = if prefix_unbalance.is_none() {
                            self.escape(&value)
                        } else {
                            value
                        };
                        if i == 0 && add_space_to_first_candidate {
                            new_value = format!(" {new_value}")
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
                .map(|(value, description, nospace, comp_color)| {
                    let new_value = self.combine_value(prefix, &value);
                    let display = if value.is_empty() { " ".into() } else { value };
                    let description = self.comp_description(&description, "", "");
                    let space: &str = if nospace { "0" } else { "1" };
                    let color = self.color(comp_color, no_color);
                    format!("{new_value}\t{space}\t{display}\t{description}\t{color}")
                })
                .collect::<Vec<String>>(),
            Shell::Fish => candidates
                .into_iter()
                .map(|(value, description, _nospace, _comp_color)| {
                    let new_value = self.combine_value(prefix, &value);
                    let description = self.comp_description(&description, "\t", "");
                    format!("{new_value}{description}")
                })
                .collect::<Vec<String>>(),
            Shell::Generic => candidates
                .into_iter()
                .map(|(value, description, nospace, comp_color)| {
                    let comp_color = format!("\t/color:{}", comp_color.ser());
                    let description = self.comp_description(&description, "\t", "");
                    let space: &str = if nospace { "\0" } else { "" };
                    format!("{value}{space}{comp_color}{description}")
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
                .map(|(value, description, nospace, comp_color)| {
                    let new_value = if let Some((ch, i)) = unbalance_quote(prefix) {
                        if i == 0 {
                            format!("{}{value}", &prefix[1..])
                        } else {
                            format!(
                                "{}{ch}{}{value}",
                                self.escape(&prefix[0..i]),
                                &prefix[i + 1..]
                            )
                        }
                    } else {
                        self.escape(&format!("{prefix}{value}"))
                    };

                    let new_value = new_value.replace(':', "\\:");
                    let display = value.replace(':', "\\:");
                    let description = self.comp_description(&description, ":", "");
                    let color = self.color(comp_color, no_color);
                    let space = if nospace { "" } else { " " };
                    format!("{new_value}{space}\t{display}{description}\t{value}\t{color}")
                })
                .collect::<Vec<String>>(),
            Shell::Tcsh => {
                if candidates.len() == 1 {
                    let new_value =
                        Self::sanitize_tcsh_value(&self.combine_value(prefix, &candidates[0].0));
                    return vec![new_value];
                }
                candidates
                    .into_iter()
                    .map(|(value, description, _, _)| {
                        let new_value =
                            Self::sanitize_tcsh_value(&self.combine_value(prefix, &value));
                        let description = self.comp_description(&description, " (", ")");
                        let description = description.replace(' ', "⠀");
                        format!("{new_value}{description}")
                    })
                    .collect::<Vec<String>>()
            }
        }
    }

    pub(crate) fn need_description<T: Runtime>(&self, runtime: T) -> bool {
        match runtime.env_var("ARGC_COMPGEN_DESCRIPTION") {
            Some(v) => is_true_value(&v),
            None => true,
        }
    }

    pub(crate) fn need_escape_chars(&self) -> &[(char, u8)] {
        // Flags:
        // 1: escape-first-char
        // 2: escape-middle-char
        // 4: escape-last-char
        match self {
            Shell::Bash => &[
                (' ', 7),
                ('!', 3),
                ('"', 7),
                ('#', 1),
                ('$', 3),
                ('&', 7),
                ('\'', 7),
                ('(', 7),
                (')', 7),
                (';', 7),
                ('<', 7),
                ('>', 7),
                ('\\', 7),
                ('`', 7),
                ('|', 7),
            ],
            Shell::Elvish => &[],
            Shell::Fish => &[],
            Shell::Generic => &[],
            Shell::Nushell => &[
                (' ', 7),
                ('!', 1),
                ('"', 7),
                ('#', 1),
                ('$', 1),
                ('\'', 7),
                ('(', 7),
                (')', 7),
                (';', 7),
                ('[', 7),
                ('`', 7),
                ('{', 7),
                ('|', 7),
                ('}', 7),
            ],
            Shell::Powershell => &[
                (' ', 7),
                ('"', 7),
                ('#', 1),
                ('$', 3),
                ('&', 7),
                ('\'', 7),
                ('(', 7),
                (')', 7),
                (',', 7),
                (';', 7),
                ('<', 1),
                ('>', 1),
                ('@', 1),
                (']', 1),
                ('`', 7),
                ('{', 7),
                ('|', 7),
                ('}', 7),
            ],
            Shell::Xonsh => &[
                (' ', 7),
                ('!', 7),
                ('"', 7),
                ('#', 7),
                ('$', 4),
                ('&', 7),
                ('\'', 7),
                ('(', 7),
                (')', 7),
                ('*', 7),
                (':', 1),
                (';', 7),
                ('<', 7),
                ('=', 1),
                ('>', 7),
                ('[', 7),
                ('\\', 4),
                (']', 7),
                ('^', 1),
                ('`', 7),
                ('{', 7),
                ('|', 7),
                ('}', 7),
            ],
            Shell::Zsh => &[
                (' ', 7),
                ('!', 3),
                ('"', 7),
                ('#', 1),
                ('$', 3),
                ('&', 7),
                ('\'', 7),
                ('(', 7),
                (')', 7),
                ('*', 7),
                (';', 7),
                ('<', 7),
                ('=', 1),
                ('>', 7),
                ('?', 7),
                ('[', 7),
                ('\\', 7),
                ('`', 7),
                ('|', 7),
            ],
            Shell::Tcsh => &[
                (' ', 7),
                ('!', 3),
                ('"', 7),
                ('$', 3),
                ('&', 7),
                ('\'', 7),
                ('(', 7),
                (')', 7),
                ('*', 7),
                (';', 7),
                ('<', 7),
                ('>', 7),
                ('?', 7),
                ('\\', 7),
                ('`', 7),
                ('{', 7),
                ('|', 7),
            ],
        }
    }

    pub(crate) fn escape(&self, value: &str) -> String {
        match self {
            Shell::Bash | Shell::Tcsh => Self::escape_chars(value, self.need_escape_chars(), "\\"),
            Shell::Elvish | Shell::Fish | Shell::Generic => value.into(),
            Shell::Nushell | Shell::Powershell | Shell::Xonsh => {
                if Self::contains_escape_chars(value, self.need_escape_chars()) {
                    format!("'{value}'")
                } else {
                    value.into()
                }
            }
            Shell::Zsh => Self::escape_chars(value, self.need_escape_chars(), "\\\\"),
        }
    }

    pub(crate) fn color(&self, comp_color: CompColor, no_color: bool) -> String {
        match self {
            Shell::Elvish => {
                if no_color {
                    return "default".into();
                }
                comp_color.style()
            }
            Shell::Powershell | Shell::Zsh => {
                if no_color {
                    return "39".into();
                }
                comp_color.ansi_code()
            }
            _ => String::new(),
        }
    }

    pub(crate) fn redirect_symbols(&self) -> Vec<&str> {
        match self {
            Shell::Nushell => vec![],
            _ => vec!["<", ">"],
        }
    }

    pub(crate) fn is_windows_mode<T: Runtime>(&self, runtime: T) -> bool {
        if runtime.is_windows() {
            !self.is_unix_only()
        } else {
            false
        }
    }

    pub(crate) fn need_break_chars<T: Runtime>(&self, runtime: T, last_arg: &str) -> Vec<char> {
        if last_arg.starts_with(is_quote_char) {
            return vec![];
        }
        match self {
            Shell::Bash => match runtime.env_var("COMP_WORDBREAKS") {
                Some(v) => [':', '=', '@']
                    .iter()
                    .filter(|c| v.contains(**c))
                    .copied()
                    .collect(),
                None => [':', '=', '@'].to_vec(),
            },
            Shell::Powershell => vec![','],
            _ => vec![],
        }
    }

    pub(crate) fn need_expand_tilde<T: Runtime>(&self, runtime: T) -> bool {
        self.is_windows_mode(runtime) && self == &Self::Powershell
    }

    fn combine_value(&self, prefix: &str, value: &str) -> String {
        let mut new_value = format!("{prefix}{value}");
        if unbalance_quote(prefix).is_none() {
            new_value = self.escape(&new_value);
        }
        new_value
    }

    fn comp_description(&self, description: &str, prefix: &str, suffix: &str) -> String {
        if description.is_empty() {
            String::new()
        } else {
            format!(
                "{prefix}{}{suffix}",
                Self::truncate_description(description)
            )
        }
    }

    fn truncate_description(description: &str) -> String {
        let max_width = 80;
        let mut description = description.trim().replace('\t', "");
        if description.starts_with('(') && description.ends_with(')') {
            description = description
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_string();
        }
        if description.chars().count() < max_width {
            description
        } else {
            let truncated: String = description.chars().take(max_width).collect();
            format!("{truncated}...")
        }
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

    fn escape_chars(value: &str, need_escape_chars: &[(char, u8)], for_escape: &str) -> String {
        let chars: Vec<char> = value.chars().collect();
        let len = chars.len();
        chars
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                if Self::match_escape_chars(need_escape_chars, c, i, len) {
                    format!("{for_escape}{c}")
                } else {
                    c.to_string()
                }
            })
            .collect()
    }

    fn contains_escape_chars(value: &str, need_escape_chars: &[(char, u8)]) -> bool {
        let chars: Vec<char> = value.chars().collect();
        chars
            .iter()
            .enumerate()
            .any(|(i, c)| Self::match_escape_chars(need_escape_chars, *c, i, chars.len()))
    }

    fn match_escape_chars(need_escape_chars: &[(char, u8)], c: char, i: usize, len: usize) -> bool {
        need_escape_chars.iter().any(|(ch, flag)| {
            if *ch == c {
                if i == 0 {
                    (*flag & 1) != 0
                } else if i == len - 1 {
                    (*flag & 4) != 0
                } else {
                    (*flag & 2) != 0
                }
            } else {
                false
            }
        })
    }

    fn sanitize_tcsh_value(value: &str) -> String {
        value.replace(' ', "⠀")
    }
}
