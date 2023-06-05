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
    line: &str,
) -> Result<String> {
    let (last_word, unbalance_char) = get_last_word(line);
    let line = if let Some(c) = unbalance_char {
        format!("{}{}", line, c)
    } else {
        line.to_string()
    };
    let mut args = split_shell_words(&line)?;
    if last_word.is_empty() {
        args.push("".into());
    }
    if args.len() == 1 {
        return Ok(String::new());
    }
    let cmd = Command::new(script_content)?;
    let matcher = Matcher::new(&cmd, &args);
    let has_prefix =
        !matcher.has_dashdash() && last_word.starts_with('-') && last_word.ends_with('=');
    let filter = if has_prefix { "" } else { &last_word };
    let candicates = matcher.compgen();
    let mut candicates = expand_candicates(candicates, script_path, &args, filter)?;
    if has_prefix {
        candicates = candicates
            .into_iter()
            .map(|(k, v)| (format!("{last_word}{k}"), v))
            .collect();
    }
    shell.convert(&candicates)
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
                Shell::list()
            ),
        }
    }
}

impl Shell {
    pub fn list() -> &'static str {
        "bash,zsh,powershell,fish,elvish,nushell"
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
                return convert_arg_value(value);
            } else {
                return value.to_string();
            }
        }
        match self {
            Shell::Bash => bash_escape(value),
            Shell::Zsh => zsh_escape(value),
            Shell::Powershell => format!("{} ", powershell_escape(value)),
            _ => value.to_string(),
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
}

fn expand_candicates(
    values: Vec<(String, String)>,
    script_file: &str,
    args: &[String],
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
        if let Some(param_fn_outputs) = run_param_fns(script_file, &fns, args) {
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
            } else if value_name.contains("arg") || value_name.contains("any") {
                output[0] = ("__argc_comp:file".into(), String::new());
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
