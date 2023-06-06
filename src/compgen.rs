use crate::command::Command;
use crate::matcher::Matcher;
use crate::utils::escape_shell_words;
use crate::utils::run_param_fns;
use crate::Result;

use anyhow::bail;
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
    let candicates = matcher.compgen();
    let mapped_candicates = mapping_candicates(&candicates, script_path, &args, shell, last)?;
    shell.convert(&mapped_candicates)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Powershell,
    Fish,
    Elvish,
    Nushell,
}

impl FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "powershell" => Ok(Self::Powershell),
            "fish" => Ok(Self::Fish),
            "elvish" => Ok(Self::Elvish),
            "nushell" => Ok(Self::Nushell),
            _ => bail!(
                "The provided shell is either invalid or missing, must be one of {}",
                Shell::list_names(),
            ),
        }
    }
}

impl Shell {
    pub fn list() -> [Shell; 6] {
        [
            Shell::Bash,
            Shell::Zsh,
            Shell::Powershell,
            Shell::Fish,
            Shell::Elvish,
            Shell::Nushell,
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
            Shell::Zsh => "zsh",
            Shell::Powershell => "powershell",
            Shell::Fish => "fish",
            Shell::Elvish => "elvish",
            Shell::Nushell => "nushell",
        }
    }

    pub fn convert(&self, candicates: &[(String, String)]) -> Result<String> {
        if candicates.len() == 1 {
            return Ok(self.convert_value(&candicates[0].0));
        }
        let need_description = self.compgen_description();
        let mut max_width = 0;
        let values: Vec<String> = candicates
            .iter()
            .map(|(v, _)| {
                let value = self.convert_value(v);
                max_width = max_width.max(value.len());
                value
            })
            .collect();
        let value_width = 95 - max_width;
        let output = values
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                let description = &candicates[i].1;
                if !need_description || description.is_empty() {
                    return value;
                }
                match self {
                    Shell::Bash => {
                        let description = if description.len() >= value_width {
                            format!("{}...", &description[..value_width])
                        } else {
                            description.clone()
                        };
                        format!("{:<width$}({})", value, description, width = max_width + 2)
                    }
                    Shell::Zsh => format!("{}:{}", value, description),
                    _ => format!("{}\t{}", value, description),
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        Ok(output)
    }

    pub fn convert_value(&self, value: &str) -> String {
        if value.starts_with("__argc_") {
            if value.starts_with("__argc_value") {
                if let Some(stripped_value) = value.strip_prefix("__argc_value") {
                    let (mark, value) = stripped_value.split_at(1);
                    match mark {
                        "+" => format!("<{value}>..."),
                        "*" => format!("[{value}]..."),
                        "!" => format!("<{value}>"),
                        ":" => format!("[{value}]"),
                        _ => value.to_string(),
                    }
                } else {
                    value.to_string()
                }
            } else {
                value.to_string()
            }
        } else {
            self.escape(value)
        }
    }

    pub fn compgen_description(&self) -> bool {
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

    pub fn escape(&self, value: &str) -> String {
        match self {
            Shell::Bash => value
                .chars()
                .map(|v| {
                    if matches!(
                        v,
                        ' ' | '!'
                            | '"'
                            | '$'
                            | '&'
                            | '\''
                            | '<'
                            | '>'
                            | '`'
                            | '|'
                            | '{'
                            | '}'
                            | '['
                            | ']'
                            | '^'
                            | '~'
                            | '#'
                            | '*'
                            | '?'
                    ) {
                        format!("\\{v}")
                    } else {
                        v.to_string()
                    }
                })
                .collect::<String>(),
            Shell::Zsh => value
                .chars()
                .map(|v| {
                    if v == ':' {
                        format!("\\{v}")
                    } else {
                        v.to_string()
                    }
                })
                .collect::<String>(),
            Shell::Powershell => escape_shell_words(value),
            _ => value.into(),
        }
    }

    pub fn word_breaks(&self) -> Vec<char> {
        match self {
            Shell::Bash => match std::env::var("COMP_WORDBREAKS") {
                Ok(v) => v.chars().collect(),
                Err(_) => vec!['=', ':'],
            },
            _ => vec![],
        }
    }
}

fn mapping_candicates(
    values: &[(String, String)],
    script_file: &str,
    args: &[String],
    shell: Shell,
    last: &str,
) -> Result<Vec<(String, String)>> {
    let mut output: Vec<(String, String)> = vec![];
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
    for (value, describe) in values {
        if let Some(param_fn) = value.strip_prefix("__argc_fn:") {
            param_fns.push(param_fn.to_string());
        } else if value.starts_with("__argc_") {
            output.push((value.to_string(), describe.to_string()));
        } else if let Some(value) = mapper(value) {
            output.push((value, describe.to_string()));
        }
    }
    if !param_fns.is_empty() {
        let fns: Vec<&str> = param_fns.iter().map(|v| v.as_str()).collect();
        if let Some(param_fn_outputs) = run_param_fns(script_file, &fns, args) {
            for param_fn_output in param_fn_outputs {
                for line in param_fn_output.split('\n') {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let (value, describe) = match line.split_once('\t') {
                        Some(v) => v,
                        None => (line, ""),
                    };
                    if value.starts_with("__argc_") {
                        output.push((value.to_string(), describe.to_string()));
                    } else if let Some(value) = mapper(value) {
                        output.push((value, describe.to_string()));
                    }
                }
            }
        }
    }

    if output.len() == 1 {
        let value = &output[0].0;
        if let Some(value_name) = value.strip_prefix("__argc_value") {
            let value_name = value_name.to_ascii_lowercase();
            if value_name.contains("path") || value_name.contains("file") {
                output[0] = ("__argc_comp:file".into(), String::new());
            } else if value_name.contains("dir") || value_name.contains("folder") {
                output[0] = ("__argc_comp:dir".into(), String::new());
            } else if value_name.contains("arg") || value_name.contains("any") {
                output[0] = ("__argc_comp:file".into(), String::new());
            } else {
                output.clear();
            };
        }
    }
    Ok(output)
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
