use crate::command::Command;
use crate::matcher::Matcher;
use crate::utils::{escape_shell_words, get_current_dir, run_param_fns};
use crate::Result;

use anyhow::bail;
use std::collections::HashMap;
use std::str::FromStr;

const MAX_DESCRIPTION_WIDTH: usize = 80;

pub fn compgen(
    shell: Shell,
    script_path: &str,
    script_content: &str,
    args: &[String],
) -> Result<String> {
    if args.len() < 2 {
        return Ok(String::new());
    }
    let (last, _) = unbalance_quote(&args[args.len() - 1]);
    let cmd = Command::new(script_content)?;
    let args: Vec<String> = args
        .iter()
        .enumerate()
        .map(|(i, v)| {
            if i == args.len() - 1 {
                last.to_string()
            } else {
                v.to_string()
            }
        })
        .collect();
    let matcher = Matcher::new(&cmd, &args);
    let compgen_values = matcher.compgen();
    let with_description = shell.with_description();
    let mut candicates: Vec<(String, bool, String)> = vec![];
    let mut param_fns = vec![];
    let mut assign_option = None;
    if args.iter().all(|v| v != "--") {
        assign_option = split_equal_sign(last);
    }
    let breaks = &shell.word_breaks();
    let mapper = |value: &str| -> Option<String> {
        if let Some((prefix, matches)) = assign_option {
            if let Some(breaked_value) = split_with_breaks(value, matches, breaks) {
                if breaked_value != value || breaks.contains(&'=') {
                    return Some(breaked_value.to_string());
                } else {
                    return Some(format!("{prefix}{value}"));
                }
            }
        }
        if let Some(value) = split_with_breaks(value, last, breaks) {
            return Some(value.to_string());
        }
        None
    };
    for (value, description) in compgen_values {
        if let Some(param_fn) = value.strip_prefix("__argc_fn:") {
            param_fns.push(param_fn.to_string());
        } else if value.starts_with("__argc_") {
            candicates.push((value.to_string(), false, description));
        } else if let Some(value) = mapper(&value) {
            candicates.push((value, false, description));
        }
    }
    if !param_fns.is_empty() {
        let mut envs = HashMap::new();
        envs.insert("ARGC_DESCRIBE".into(), with_description.to_string());
        if let Some(cwd) = get_current_dir() {
            envs.insert("ARGC_PWD".into(), escape_shell_words(&cwd));
        }
        let fns: Vec<&str> = param_fns.iter().map(|v| v.as_str()).collect();
        if let Some(list) = run_param_fns(script_path, &fns, &args, envs) {
            for lines in list {
                for line in lines {
                    let (value, description) = line.split_once('\t').unwrap_or((line.as_str(), ""));
                    let (value, nospace) = match value.strip_suffix('\0') {
                        Some(value) => (value, true),
                        None => (value, false),
                    };
                    if value.starts_with("__argc_") {
                        candicates.push((value.to_string(), false, description.to_string()));
                    } else if let Some(value) = mapper(value) {
                        candicates.push((value, nospace, description.to_string()));
                    }
                }
            }
        }
    }
    Ok(shell.convert_candicates(&candicates, with_description))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
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
            Shell::Nushell => "nushell",
            Shell::Powershell => "powershell",
            Shell::Xonsh => "xonsh",
            Shell::Zsh => "zsh",
        }
    }

    pub fn convert_candicates(
        &self,
        candicates: &[(String, bool, String)],
        with_description: bool,
    ) -> String {
        if candicates.len() == 1 {
            if let Some(value_name) = candicates[0].0.strip_prefix("__argc_value") {
                let value_name = value_name.to_lowercase();
                if value_name.contains("path")
                    || value_name.contains("file")
                    || value_name.contains("arg")
                    || value_name.contains("any")
                {
                    return "__argc_comp:file".into();
                } else if value_name.contains("dir") || value_name.contains("folder") {
                    return "__argc_comp:dir".into();
                } else {
                    return String::new();
                }
            }
        }
        let iter = candicates.iter().map(|(value, nospace, description)| {
            let value = self.convert_value(value);
            let description = truncate_description(description, MAX_DESCRIPTION_WIDTH);
            (value, *nospace, description)
        });
        match self {
            Shell::Bash => iter
                .map(|(value, nospace, description)| {
                    if nospace {
                        value
                    } else if description.is_empty() || !with_description {
                        format!("{value} ")
                    } else {
                        format!("{value} ({description})")
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Fish => iter
                .map(|(value, _, description)| {
                    if description.is_empty() || !with_description {
                        value
                    } else {
                        format!("{value}\t{description}")
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Nushell => iter
                .map(|(value, nospace, description)| {
                    let space = if nospace { "" } else { " " };
                    if description.is_empty() || !with_description {
                        format!("{value}{space}")
                    } else {
                        format!("{value}{space}\t{description}")
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            _ => iter
                .map(|(value, nospace, description)| {
                    let space: &str = if nospace { "0" } else { "1" };
                    format!("{value}\t{space}\t{description}")
                })
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }

    pub(crate) fn convert_value(&self, value: &str) -> String {
        if value.starts_with("__argc_") {
            if value.starts_with("__argc_value") {
                if let Some(stripped_value) = value.strip_prefix("__argc_value") {
                    let (mark, value) = stripped_value.split_at(1);
                    return match mark {
                        "+" => format!("<{value}>..."),
                        "*" => format!("[{value}]..."),
                        "!" => format!("<{value}>"),
                        ":" => format!("[{value}]"),
                        _ => value.to_string(),
                    };
                }
            }
            value.to_string()
        } else {
            self.escape(value)
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
        if self == &Shell::Bash {
            return false;
        }
        true
    }

    pub(crate) fn escape(&self, value: &str) -> String {
        match self {
            Shell::Bash => escape_chars(value, r###"()<>"'` !#$&;\|"###),
            Shell::Nushell => {
                if contains_chars(value, r###"()[]{}"'` #$;|"###) {
                    let value = escape_chars(value, "\"");
                    format!("\"{value}\"")
                } else {
                    value.into()
                }
            }
            Shell::Powershell => {
                if contains_chars(value, r###"()<>[]{}"'` #$&,;@|"###) {
                    let value: String = value
                        .chars()
                        .map(|c| {
                            if c == '\'' {
                                "''".to_string()
                            } else {
                                c.to_string()
                            }
                        })
                        .collect();
                    format!("'{value}'")
                } else {
                    value.into()
                }
            }
            Shell::Xonsh => {
                if contains_chars(value, r###"()<>[]{}!"'` #&:;\|"###) {
                    let value = escape_chars(value, "'");
                    format!("'{value}'")
                } else {
                    value.into()
                }
            }
            Shell::Zsh => escape_chars(value, r###"()<>[]"'` !#$&*;?\|~"###),
            _ => value.into(),
        }
    }

    pub(crate) fn word_breaks(&self) -> Vec<char> {
        match self {
            Shell::Bash => match std::env::var("COMP_WORDBREAKS") {
                Ok(v) => v.chars().collect(),
                Err(_) => vec!['=', ':'],
            },
            _ => vec![],
        }
    }
}

fn unbalance_quote(arg: &str) -> (&str, Option<char>) {
    if arg.starts_with(is_quote) && arg.chars().skip(1).all(|v| !is_quote(v)) {
        return (&arg[1..], arg.chars().next());
    }
    (arg, None)
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

fn split_with_breaks<'a>(value: &'a str, matches: &str, breaks: &[char]) -> Option<&'a str> {
    if !value.starts_with(matches) {
        return None;
    }
    if let Some(idx) = matches.rfind(|c| breaks.contains(&c)) {
        return Some(&value[idx + 1..]);
    }
    Some(value)
}

fn escape_chars(value: &str, chars: &str) -> String {
    let chars: Vec<char> = chars.chars().collect();
    value
        .chars()
        .map(|c| {
            if chars.contains(&c) {
                format!("\\{c}")
            } else {
                c.to_string()
            }
        })
        .collect()
}

fn truncate_description(description: &str, max_width: usize) -> String {
    let description = description.trim().replace('\t', "");
    if description.len() < max_width {
        description
    } else {
        format!("{}...", &description[0..max_width - 3])
    }
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

    #[test]
    fn test_split_equal_sign() {
        assert_eq!(split_equal_sign("-a="), Some(("-a=", "")));
        assert_eq!(split_equal_sign("-a="), Some(("-a=", "")));
        assert_eq!(split_equal_sign("a"), None);
        assert_eq!(split_equal_sign("a:"), None);
        assert_eq!(split_equal_sign("=a"), None);
    }

    #[test]
    fn test_split_with_breaks() {
        assert_eq!(split_with_breaks("abc", "b", &[]), None);
        assert_eq!(split_with_breaks("abc", "", &[]), Some("abc"));
        assert_eq!(split_with_breaks("abc:", "abc:", &[]), Some("abc:"));
        assert_eq!(split_with_breaks("abc:", "", &[':']), Some("abc:"));
        assert_eq!(split_with_breaks("abc:", "abc:", &[':']), Some(""));
        assert_eq!(split_with_breaks("abc:def", "abc:", &[':']), Some("def"));
    }
}
