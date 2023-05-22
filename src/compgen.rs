use crate::command::Command;
use crate::matcher::Matcher;
use crate::utils::{escape_shell_words, run_param_fns, split_shell_words};
use crate::Result;
use anyhow::bail;
use std::str::FromStr;

pub fn compgen(
    shell: Shell,
    script_path: &str,
    script_content: &str,
    bin_name: &str,
    line: &str,
) -> Result<String> {
    let (last_word, unbalance_char) = get_last_word(line);
    let line = if let Some(c) = unbalance_char {
        format!("{}{}", line, c)
    } else {
        line.to_string()
    };
    let mut args = split_shell_words(&line)?;
    args.insert(0, bin_name.to_string());
    if line.trim_end() != line {
        args.push("".into());
    }
    let cmd = Command::new(script_content)?;
    let matcher = Matcher::new(&cmd, &args);
    let candicates = matcher.compgen();
    let candicates = expand_candicates(candicates, script_path, &line, &last_word)?;
    shell.convert(&candicates, &last_word)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Powershell,
    Fish,
    Elvish,
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
            _ => bail!(
                "The provided shell is either invalid or missing, must be one of {}",
                Shell::list()
            ),
        }
    }
}

impl Shell {
    pub fn list() -> &'static str {
        "bash,zsh,powershell,fish"
    }

    pub fn convert(&self, candicates: &[(String, String)], last_word: &str) -> Result<String> {
        if candicates.len() == 1 {
            return Ok(self.convert_value(&candicates[0].0, last_word));
        }
        let with_description = self.with_description();
        let mut max_width = 0;
        let values: Vec<String> = candicates
            .iter()
            .map(|(v, _)| {
                let value = self.convert_value(v, last_word);
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
                if !with_description || description.is_empty() {
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
                    Shell::Powershell => format!("{}\t{}", value, description),
                    Shell::Fish | Shell::Elvish => format!("{}\t{}", value, description),
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        Ok(output)
    }

    pub fn convert_value(&self, value: &str, last_word: &str) -> String {
        if value.starts_with("__argc_") {
            if value.starts_with("__argc_value") {
                return convert_arg_value(value);
            } else {
                return value.to_string();
            }
        }
        match self {
            Shell::Bash => {
                if let Some((prefix, _)) =
                    last_word.rsplit_once(|c| self.word_breaks().contains(&c))
                {
                    if let Some(value) = value.strip_prefix(&last_word[0..prefix.len() + 1]) {
                        return value.to_string();
                    }
                }
                bash_escape(value)
            }
            Shell::Zsh => zsh_escape(value),
            Shell::Powershell => format!("{} ", powershell_escape(value)),
            Shell::Fish | Shell::Elvish => value.to_string(),
        }
    }

    pub fn word_breaks(&self) -> Vec<char> {
        match self {
            Shell::Bash => match std::env::var("COMP_WORDBREAKS") {
                Ok(v) => v.chars().collect(),
                Err(_) => vec!['=', ':', '|'],
            },
            _ => vec![],
        }
    }

    pub fn with_description(&self) -> bool {
        if let Ok(v) = std::env::var("ARGC_COMPLETION_DESCRIPTION") {
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
}

fn expand_candicates(
    values: Vec<(String, String)>,
    script_file: &str,
    line: &str,
    filter: &str,
) -> Result<Vec<(String, String)>> {
    let mut output = vec![];
    let mut param_fns = vec![];
    for (value, describe) in values {
        if let Some(param_fn) = value.strip_prefix("__argc_fn:") {
            param_fns.push(param_fn.to_string());
        } else if value.starts_with("__argc_") || value.starts_with(filter) {
            output.push((value, describe));
        }
    }
    if !param_fns.is_empty() {
        let fns: Vec<&str> = param_fns.iter().map(|v| v.as_str()).collect();
        if let Some(param_fn_outputs) = run_param_fns(script_file, &fns, line) {
            for param_fn_output in param_fn_outputs {
                for output_line in param_fn_output.split('\n') {
                    let output_line = output_line.trim();
                    if !output_line.is_empty()
                        && (output_line.starts_with("__argc_") || output_line.starts_with(filter))
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

    if output.len() == 1 {
        let value = &output[0].0;
        if let Some(value_name) = value.strip_prefix("__argc_value") {
            let value_name = value_name.to_ascii_lowercase();
            if value_name.contains("path") || value_name.contains("file") {
                output[0] = ("__argc_comp:file".into(), String::new());
            } else if value_name.contains("dir") || value_name.contains("folder") {
                output[0] = ("__argc_comp:dir".into(), String::new());
            } else {
                output.clear();
            };
        }
    }
    Ok(output)
}

fn get_last_word(line: &str) -> (String, Option<char>) {
    let mut word = vec![];
    let mut balances = vec![];
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' {
            if i < chars.len() - 1 {
                i += 1;
                word.push(chars[i]);
            }
        } else if c.is_ascii_whitespace() {
            if balances.is_empty() {
                word.clear();
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
        i += 1
    }
    if word.is_empty() {
        return (String::new(), None);
    }
    if balances.is_empty() {
        if word[0] == '\'' || word[0] == '\"' {
            return (word[1..word.len() - 1].iter().collect(), None);
        }
        return (word.into_iter().collect(), None);
    }
    (word[1..].iter().collect(), Some(word[0]))
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
        .collect::<String>()
}

fn powershell_escape(value: &str) -> String {
    escape_shell_words(value)
}

fn convert_arg_value(name: &str) -> String {
    if let Some(value_name) = name.strip_prefix("__argc_value") {
        let (mark, value) = value_name.split_at(1);
        match mark {
            "+" => format!("<{value}>..."),
            "*" => format!("[{value}]..."),
            "!" => format!("<{value}>"),
            ":" => format!("[{value}]"),
            _ => name.to_string(),
        }
    } else {
        name.to_string()
    }
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
        assert_eq!(get_last_word("foo\\ def").0, "foo def");
    }
}
