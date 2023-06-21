use crate::command::Command;
use crate::matcher::Matcher;
use crate::utils::{escape_shell_words, get_current_dir, run_param_fns};
use crate::Result;

use anyhow::bail;
use indexmap::IndexMap;
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
    let (mut last, _) = unbalance_quote(&args[args.len() - 1]);
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
    let mut prefix = "";
    let mut candidates: IndexMap<String, (String, bool)> = IndexMap::new();
    let mut argc_fn = None;
    let mut argc_value = None;
    if args.iter().all(|v| v != "--") {
        if let Some((left, right)) = split_equal_sign(last) {
            prefix = left;
            last = right
        }
    }
    for (value, description) in compgen_values {
        if value.starts_with("__argc_") {
            if let Some(fn_name) = value.strip_prefix("__argc_fn:") {
                argc_fn = Some(fn_name.to_string());
            } else if let Some(value) = value.strip_prefix("__argc_value:") {
                argc_value = argc_value.or_else(|| Some(value.to_string()));
            }
        } else if value.starts_with(last) {
            candidates.insert(value.clone(), (description, false));
        }
    }
    let mut argc_prefix = prefix.to_string();
    let mut argc_matcher = last.to_string();
    if let Some(fn_name) = argc_fn {
        let mut envs = HashMap::new();
        let with_description = shell.with_description();
        envs.insert("ARGC_DESCRIBE".into(), with_description.to_string());
        envs.insert("ARGC_MATCHER".into(), argc_matcher.clone());
        if let Some(cwd) = get_current_dir() {
            envs.insert("ARGC_PWD".into(), escape_shell_words(&cwd));
        }
        if let Some(outputs) = run_param_fns(script_path, &[fn_name.as_str()], &args, envs) {
            for line in outputs[0].trim().split('\n').map(|v| v.trim()) {
                let (value, description) = line.split_once('\t').unwrap_or((line, ""));
                let (value, nospace) = match value.strip_suffix('\0') {
                    Some(value) => (value, true),
                    None => (value, false),
                };
                if value.starts_with("__argc_") {
                    if let Some(value) = value.strip_prefix("__argc_value:") {
                        argc_value = argc_value.or_else(|| Some(value.to_string()));
                    } else if let Some(value) = value.strip_prefix("__argc_comp:") {
                        argc_value = Some(value.to_string());
                    } else if let Some(value) = value.strip_prefix("__argc_prefix:") {
                        argc_prefix = format!("{prefix}{value}")
                    } else if let Some(value) = value.strip_prefix("__argc_matcher:") {
                        argc_matcher = value.to_string();
                    }
                } else if value.starts_with(&argc_matcher) {
                    match candidates.get_mut(value) {
                        Some((v1, v2)) => {
                            if v1.is_empty() && !description.is_empty() {
                                *v1 = description.to_string();
                            }
                            if !*v2 && nospace {
                                *v2 = true
                            }
                        }
                        None => {
                            candidates
                                .insert(value.to_string(), (description.to_string(), nospace));
                        }
                    }
                }
            }
        }
    }
    if candidates.is_empty() {
        if let Some(value) = argc_value {
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
    let mut candidates: Vec<(String, String, bool)> = candidates
        .into_iter()
        .map(|(value, (description, nospace))| (value, description, nospace))
        .collect();
    if shell == Shell::Bash {
        let break_chars = match std::env::var("COMP_WORDBREAKS") {
            Ok(v) => v.chars().collect(),
            Err(_) => vec!['=', ':', '|', ';'],
        };
        if argc_prefix == prefix && prefix.ends_with(|c| break_chars.contains(&c)) {
            argc_prefix = String::new();
        }
        if last == argc_matcher {
            if let Some((i, _)) = argc_matcher
                .char_indices()
                .rfind(|(_, c)| break_chars.contains(c))
            {
                argc_prefix = String::new();
                let idx = i + 1;
                argc_matcher = argc_matcher[idx..].to_string();
                for (value, _, _) in candidates.iter_mut() {
                    *value = value[idx..].to_string()
                }
            };
        }
    }
    Ok(shell.output_candidates(candidates, &argc_prefix, &argc_matcher))
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

    pub(crate) fn output_candidates(
        &self,
        candidates: Vec<(String, String, bool)>,
        prefix: &str,
        matcher: &str,
    ) -> String {
        if candidates.is_empty() {
            return String::new();
        }
        match self {
            Shell::Bash => {
                let values: Vec<&str> = candidates.iter().map(|(v, _, _)| v.as_str()).collect();
                let values_len = values.len();
                let mut add_space_to_first = false;
                if values_len == 1 {
                    let (value, _, nospace) = &candidates[0];
                    let space = if *nospace { "" } else { " " };
                    return format!("{}{space}", self.escape(&format!("{prefix}{value}")));
                } else if let Some(common) = common_prefix(&values) {
                    if common != matcher {
                        return format!("{prefix}{}", self.escape(&common));
                    } else {
                        add_space_to_first = true;
                    }
                }
                candidates
                    .into_iter()
                    .enumerate()
                    .map(|(i, (value, description, nospace))| {
                        let mut escaped_value = self.escape(&value);
                        if i == 0 && add_space_to_first {
                            escaped_value = format!(" {}", escaped_value)
                        };
                        if nospace {
                            escaped_value
                        } else if description.is_empty() || !self.with_description() {
                            format!("{escaped_value} ")
                        } else {
                            format!("{escaped_value} ({})", truncate_description(&description))
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            Shell::Fish => candidates
                .into_iter()
                .map(|(value, description, _nospace)| {
                    let escaped_value = self.escape(&format!("{prefix}{}", value));
                    if description.is_empty() || !self.with_description() {
                        escaped_value
                    } else {
                        format!("{escaped_value}\t{}", truncate_description(&description))
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Nushell => candidates
                .into_iter()
                .map(|(value, description, nospace)| {
                    let escaped_value = self.escape(&format!("{prefix}{}", value));
                    let space = if nospace { "" } else { " " };
                    if description.is_empty() || !self.with_description() {
                        format!("{escaped_value}{space}")
                    } else {
                        format!(
                            "{escaped_value}{space}\t{}",
                            truncate_description(&description)
                        )
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Zsh => candidates
                .into_iter()
                .map(|(value, description, nospace)| {
                    let escaped_value = self.escape(&format!("{prefix}{}", value));
                    let display = self.escape(&value);
                    let description = if description.is_empty() || !self.with_description() {
                        display
                    } else {
                        format!("{display}:{}", truncate_description(&description))
                    };
                    let space = if nospace { "" } else { " " };
                    format!("{escaped_value}{space}\t{description}")
                })
                .collect::<Vec<String>>()
                .join("\n"),
            _ => candidates
                .into_iter()
                .map(|(value, description, nospace)| {
                    let escaped_value = self.escape(&format!("{prefix}{}", value));
                    let display = if value.is_empty() { " ".into() } else { value };
                    let description = if description.is_empty() || !self.with_description() {
                        String::new()
                    } else {
                        truncate_description(&description)
                    };
                    let space: &str = if nospace { "0" } else { "1" };
                    format!("{escaped_value}\t{space}\t{display}\t{description}")
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
}
