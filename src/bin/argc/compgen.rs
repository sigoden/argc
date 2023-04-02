use crate::utils::*;

use anyhow::{bail, Result};
use argc::utils::escape_shell_words;
use std::{process, str::FromStr};

pub fn generate(shell: Shell, script: &str, source: &str, args: &[&str]) -> Result<String> {
    let line = if args.len() == 1 { "" } else { args[1] };
    let (last_word, unbalance_char) = get_last_word(line);
    let line = if let Some(c) = unbalance_char {
        format!("{}{}", line, c)
    } else {
        line.to_string()
    };
    let candicates = argc::compgen(source, &line)?;
    let candicates = expand_candicates(candicates, script, &line, &last_word)?;
    shell.convert(&candicates)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Powershell,
    Fish,
}

impl FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "powershell" => Ok(Self::Powershell),
            "fish" => Ok(Self::Fish),
            _ => bail!("Invalid shell value, must be one of {}", Shell::list()),
        }
    }
}

impl Shell {
    pub fn list() -> &'static str {
        "bash,zsh,powershell,fish"
    }
    pub fn convert(&self, values: &[(String, String)]) -> Result<String> {
        let output = match self {
            Shell::Bash => values
                .iter()
                .map(|(value, _)| bash_escape(value))
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Zsh => values
                .iter()
                .map(|(value, describe)| {
                    let value = zsh_escape(value);
                    if describe.is_empty() {
                        value
                    } else {
                        format!("{}:{}", value, describe)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Powershell => values
                .iter()
                .map(|(value, _)| powershell_escape(value))
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Fish => values
                .iter()
                .map(|(value, describe)| {
                    if describe.is_empty() {
                        value.into()
                    } else {
                        format!("{}\t{}", value, describe)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
        };
        Ok(output)
    }
}

fn expand_candicates(
    values: Vec<(String, String)>,
    script_file: &str,
    line: &str,
    last_word: &str,
) -> Result<Vec<(String, String)>> {
    let mut output = vec![];
    let mut param_fns = vec![];
    for (value, describe) in values {
        if let Some(param_fn) = value.strip_prefix("__argc_fn:") {
            param_fns.push(param_fn.to_string());
        } else if value.starts_with("__argc_") || value.starts_with(last_word) {
            output.push((value, describe));
        }
    }
    if !param_fns.is_empty() {
        if let Some(shell) = get_shell_path() {
            for param_fn in param_fns {
                if let Ok(fn_output) = process::Command::new(&shell)
                    .arg(script_file)
                    .arg(&param_fn)
                    .arg(line)
                    .output()
                {
                    let fn_output = String::from_utf8_lossy(&fn_output.stdout);
                    for fn_output_line in fn_output.split('\n') {
                        let output_line = fn_output_line.trim();
                        if !output_line.is_empty()
                            && (output_line.starts_with("__argc_")
                                || output_line.starts_with(last_word))
                        {
                            if let Some((x, y)) = output_line.split_once('\t') {
                                output.push((x.to_string(), y.to_string()));
                            } else {
                                output.push((output_line.to_string(), String::new()));
                            }
                        }
                    }
                }
            }
        }
    }
    if output.len() == 1 {
        let value = &output[0].0;
        if let Some(value_name) = value.strip_prefix("__argc_value") {
            if value_name.contains("PATH") || value_name.contains("FILE") {
                output[0].0 = "__argc_comp:file".into()
            } else if value_name.contains("DIR") || value_name.contains("FOLDER") {
                output[0].0 = "__argc_comp:dir".into()
            } else {
                output.clear();
            };
        }
    } else {
        output.iter_mut().for_each(|(name, _)| {
            if let Some(value_name) = name.strip_prefix("__argc_value") {
                let (mark, value) = value_name.split_at(1);
                *name = match mark {
                    "+" => format!("<{value}>..."),
                    "*" => format!("[{value}]..."),
                    "!" => format!("<{value}>"),
                    ":" => format!("[{value}]"),
                    _ => name.to_string(),
                };
            }
        })
    }
    Ok(output)
}

fn get_last_word(line: &str) -> (String, Option<char>) {
    let mut word = vec![];
    let mut balances = vec![];
    for c in line.chars() {
        if c.is_ascii_whitespace() {
            if balances.is_empty() {
                word.clear();
                if !word.is_empty() {
                    continue;
                }
                continue;
            } else {
                word.push(c);
            }
        } else if c == '\'' || c == '"' {
            if balances.last() == Some(&c) {
                balances.pop();
            } else {
                balances.push(c);
            }
            word.push(c);
        } else {
            word.push(c);
        }
    }
    if word.is_empty() {
        return (String::new(), None);
    }
    if balances.is_empty() {
        if word[0] == '\'' || word[0] == '\"' {
            return (word[1..word.len() - 1].into_iter().collect(), None);
        }
        return (word.into_iter().collect(), None);
    }
    (word[1..].into_iter().collect(), Some(word[0]))
}

fn zsh_escape(value: &str) -> String {
    value
        .chars()
        .map(|v| {
            if v == ':' {
                format!("\\{v}")
            } else {
                v.to_string()
            }
        })
        .collect::<String>()
}

fn bash_escape(value: &str) -> String {
    value
        .chars()
        .map(|v| {
            if v.is_ascii() && !(v.is_ascii_alphanumeric() || matches!(v, '_' | '-' | '/')) {
                format!("\\{v}")
            } else {
                v.to_string()
            }
        })
        .collect::<String>()
}

fn powershell_escape(value: &str) -> String {
    escape_shell_words(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_last_word() {
        assert_eq!(get_last_word("").0, "");
        assert_eq!(get_last_word(" ").0, "");
        assert_eq!(get_last_word("foo").0, "foo");
        assert_eq!(get_last_word("foo ").0, "");
        assert_eq!(get_last_word(" foo").0, "foo");
        assert_eq!(get_last_word("'foo'").0, "foo");
        assert_eq!(get_last_word("\"foo\"").0, "foo");
        assert_eq!(get_last_word("'abc "), ("abc ".into(), Some('\'')));
        assert_eq!(get_last_word("\"abc "), ("abc ".into(), Some('"')));
    }
}
