use crate::command::Command;
use crate::matcher::Matcher;
use crate::utils::{escape_shell_words, get_current_dir, run_param_fns};
use crate::Result;

use anyhow::bail;
use std::collections::HashMap;
use std::str::FromStr;

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
    let mut candicates: Vec<Candicate> = vec![];
    let mut param_fns = vec![];
    let mut param_val = None;
    let no_dashdash = args.iter().all(|v| v != "--");
    let mut word = last;
    let mut prefix = "";
    if no_dashdash {
        if let Some((left, right)) = split_equal_sign(word) {
            word = right;
            prefix = left;
        }
    }
    for (value, description) in compgen_values {
        if value.starts_with("__argc_") {
            if let Some(param_fn) = value.strip_prefix("__argc_fn:") {
                param_fns.push(param_fn.to_string());
            } else if let Some(value) = value.strip_prefix("__argc_value:") {
                param_val = param_val.or_else(|| Some(value.to_string()));
            }
        } else if value.starts_with(word) {
            candicates.push(Candicate::new(value, description, false));
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
                        if let Some(val) = value.strip_prefix("__argc_value") {
                            param_val = param_val.or_else(|| Some(val.to_string()));
                        }
                    } else if value.starts_with(word) {
                        candicates.push(Candicate::new(
                            value.to_string(),
                            description.to_string(),
                            nospace,
                        ));
                    }
                }
            }
        }
    }
    if candicates.is_empty() {
        if let Some(value) = param_val {
            let value = value.to_lowercase();
            let output = if ["path", "file", "arg", "any"]
                .iter()
                .any(|v| value.contains(v))
            {
                "__argc_comp:file".to_string()
            } else if value.contains("dir") || value.contains("folder") {
                "__argc_comp:dir".to_string()
            } else {
                String::new()
            };
            return Ok(output);
        }
    }
    Ok(shell.output_candicates(&candicates, word, prefix, with_description))
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

    pub(crate) fn output_candicates(
        &self,
        candicates: &[Candicate],
        word: &str,
        prefix: &str,
        with_description: bool,
    ) -> String {
        match self {
            Shell::Bash => candicates
                .iter()
                .map(|candicate| {
                    let value = match strip_value(&candicate.value, word, &self.word_breaks()) {
                        Some(v) => v,
                        None => &candicate.value,
                    };
                    if candicate.nospace {
                        value.to_string()
                    } else if candicate.description.is_empty() || !with_description {
                        format!("{value} ")
                    } else {
                        format!("{value} ({})", candicate.truncate_description())
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Fish => candicates
                .iter()
                .map(|candicate| {
                    let value = self.escape(&format!("{prefix}{}", candicate.value));
                    if candicate.description.is_empty() || !with_description {
                        value
                    } else {
                        format!("{value}\t{}", candicate.truncate_description())
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Nushell => candicates
                .iter()
                .map(|candicate| {
                    let value = self.escape(&format!("{prefix}{}", candicate.value));
                    let space = if candicate.nospace { "" } else { " " };
                    if candicate.description.is_empty() || !with_description {
                        format!("{value}{space}")
                    } else {
                        format!("{value}{space}\t{}", candicate.truncate_description())
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Zsh => candicates
                .iter()
                .map(|candicate| {
                    let mut value = self.escape(&format!("{prefix}{}", candicate.value));
                    let display = self.escape(&candicate.value);

                    let description = if candicate.description.is_empty() || !with_description {
                        display
                    } else {
                        format!("{display}:{}", candicate.truncate_description())
                    };
                    if !candicate.nospace {
                        value.push(' ');
                    }
                    format!("{value}\t{description}")
                })
                .collect::<Vec<String>>()
                .join("\n"),
            _ => candicates
                .iter()
                .map(|candicate| {
                    let value = self.escape(&format!("{prefix}{}", candicate.value));
                    let display = candicate.value.to_string();
                    let description = if candicate.description.is_empty() || !with_description {
                        String::new()
                    } else {
                        candicate.truncate_description()
                    };
                    let space: &str = if candicate.nospace { "0" } else { "1" };
                    format!("{value}\t{space}\t{display}\t{description}")
                })
                .collect::<Vec<String>>()
                .join("\n"),
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
            Shell::Zsh => escape_chars(value, r###"()<>[]"'` !#$&*:;?\|~"###),
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

#[derive(Debug, Clone)]
pub(crate) struct Candicate {
    value: String,
    description: String,
    nospace: bool,
}

impl Candicate {
    pub(crate) fn new(value: String, description: String, nospace: bool) -> Self {
        Self {
            value,
            description,
            nospace,
        }
    }

    pub(crate) fn truncate_description(&self) -> String {
        let max_width = 80;
        let description = self.description.trim().replace('\t', "");
        if description.len() < max_width {
            description
        } else {
            format!("{}...", &description[0..max_width - 3])
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

fn strip_value<'a>(value: &'a str, word: &str, breaks: &[char]) -> Option<&'a str> {
    if !value.starts_with(word) {
        return None;
    }
    let hyphen = word.starts_with('-');
    let (i, _) = word.char_indices().rfind(|(_, c)| {
        if hyphen {
            *c != '-' && breaks.contains(c)
        } else {
            breaks.contains(c)
        }
    })?;
    value.strip_prefix(&word[0..=i])
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
        assert_eq!(split_equal_sign("-a=c"), Some(("-a=", "c")));
        assert_eq!(split_equal_sign("a"), None);
        assert_eq!(split_equal_sign("a:"), None);
        assert_eq!(split_equal_sign("=a"), None);
    }

    #[test]
    fn test_strip_value() {
        let breaks = vec![':', '-', '/'];
        assert_eq!(strip_value("abc:def", "abc:d", &breaks), Some("def"));
        assert_eq!(strip_value("abc:def", "abc:", &breaks), Some("def"));
        assert_eq!(strip_value("abc:def", "abc", &breaks), None);
        assert_eq!(strip_value("abc-def", "abc-d", &breaks), Some("def"));
        assert_eq!(strip_value("--abc-def", "--abc-", &breaks), None);
    }
}
