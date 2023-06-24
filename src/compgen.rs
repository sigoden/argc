use crate::command::Command;
use crate::matcher::Matcher;
use crate::utils::{escape_shell_words, get_current_dir, run_param_fns};
use crate::Result;

use anyhow::bail;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
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
    let last_arg = args[args.len() - 1].as_str();
    let (mut last, unbalance) = {
        if last_arg.starts_with(is_quote) && last_arg.chars().skip(1).all(|v| !is_quote(v)) {
            (last_arg[1..].to_string(), last_arg.chars().next())
        } else {
            (last_arg.to_string(), None)
        }
    };
    let cmd = Command::new(script_content)?;
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
    let matcher = Matcher::new(&cmd, &new_args);
    let compgen_values = matcher.compgen();
    let mut default_nospace = unbalance.is_some();
    let mut prefix = unbalance.map(|v| v.to_string()).unwrap_or_default();
    let mut candidates: IndexMap<String, (String, bool)> = IndexMap::new();
    let mut argc_fn = None;
    let mut argc_value = None;
    let mut argc_variables = vec![];
    let mut multi_values = HashSet::new();
    if args.iter().all(|v| v != "--") && last.starts_with('-') {
        if let Some((left, right)) = split_equal_sign(&last) {
            prefix.push_str(left);
            last = right.to_string();
            mod_quote(&mut last, &mut prefix, &mut default_nospace);
        }
    }
    for (value, description) in compgen_values {
        if value.starts_with("__argc_") {
            if let Some(fn_name) = value.strip_prefix("__argc_fn:") {
                argc_fn = Some(fn_name.to_string());
            } else if let Some(stripped_value) = value.strip_prefix("__argc_value:") {
                argc_value = argc_value.or_else(|| Some(stripped_value.to_string()));
                if shell.is_generic() {
                    argc_variables.push(value.to_string());
                }
            } else if let Some(value) = value.strip_prefix("__argc_multi:") {
                if let Some(ch) = value.chars().next() {
                    if ch != '*' {
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
            }
        } else if value.starts_with(&last) && !multi_values.contains(&value) {
            candidates.insert(value.clone(), (description, default_nospace));
        }
    }
    let mut argc_prefix = prefix.to_string();
    let mut argc_suffix = String::new();
    let mut argc_matcher = last.to_string();
    if let Some(fn_name) = argc_fn {
        let mut envs = HashMap::new();
        let with_description = shell.with_description();
        envs.insert("ARGC_DESCRIBE".into(), with_description.to_string());
        envs.insert("ARGC_MATCHER".into(), argc_matcher.clone());
        envs.insert("ARGC_LAST_ARG".into(), last_arg.to_string());
        if let Some(cwd) = get_current_dir() {
            envs.insert("ARGC_PWD".into(), escape_shell_words(&cwd));
        }
        if let Some(outputs) = run_param_fns(script_path, &[fn_name.as_str()], &new_args, envs) {
            if !outputs[0].is_empty() {
                for line in outputs[0].trim().split('\n').map(|v| v.trim()) {
                    let (value, description) = line.split_once('\t').unwrap_or((line, ""));
                    let (value, nospace) = match value.strip_suffix('\0') {
                        Some(value) => (value, true),
                        None => (value, false),
                    };
                    let nospace = nospace || default_nospace;
                    if value.starts_with("__argc_") {
                        if let Some(stripped_value) = value.strip_prefix("__argc_value:") {
                            argc_value = argc_value.or_else(|| Some(stripped_value.to_string()));
                        } else if let Some(stripped_value) = value.strip_prefix("__argc_prefix:") {
                            argc_prefix = format!("{prefix}{stripped_value}")
                        } else if let Some(stripped_value) = value.strip_prefix("__argc_suffix:") {
                            argc_suffix = stripped_value.to_string();
                        } else if let Some(stripped_value) = value.strip_prefix("__argc_matcher:") {
                            argc_matcher = stripped_value.to_string();
                        }
                        if shell.is_generic() {
                            argc_variables.push(value.to_string());
                        }
                    } else if value.starts_with(&argc_matcher) && !multi_values.contains(value) {
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
    }

    if !argc_variables.is_empty() {
        let mut prepend_candicates: IndexMap<String, (String, bool)> = argc_variables
            .into_iter()
            .map(|value| (value, (String::new(), false)))
            .collect();
        prepend_candicates.extend(candidates);
        candidates = prepend_candicates;
    }

    if candidates.is_empty() {
        if let Some(value) = argc_value {
            return Ok(compgen_arg_value(&value));
        }
        return Ok(String::new());
    }
    let mut candidates: Vec<(String, String, bool)> = candidates
        .into_iter()
        .map(|(value, (description, nospace))| (value, description, nospace))
        .collect();
    let break_chars = shell.need_break_chars();
    if !break_chars.is_empty() {
        let prefix_unbalance = unbalance_quote(&argc_prefix);
        if let Some((i, _)) = (match prefix_unbalance {
            Some((_, idx)) => &argc_prefix[0..idx],
            None => &argc_prefix,
        })
        .char_indices()
        .rfind(|(_, c)| break_chars.contains(c))
        {
            argc_prefix = argc_prefix[i + 1..].to_string();
        }
        if last == argc_matcher && prefix_unbalance.is_none() {
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
    Ok(shell.output_candidates(candidates, &argc_prefix, &argc_suffix, &argc_matcher))
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

    pub(crate) fn output_candidates(
        &self,
        candidates: Vec<(String, String, bool)>,
        prefix: &str,
        suffix: &str,
        matcher: &str,
    ) -> String {
        match self {
            Shell::Bash => {
                let prefix_unbalance = unbalance_quote(prefix);
                let values: Vec<&str> = candidates.iter().map(|(v, _, _)| v.as_str()).collect();
                let mut add_space_to_first_candidate = false;
                if values.len() == 1 {
                    let space = if candidates[0].2 { "" } else { " " };
                    let mut value = format!("{prefix}{}{suffix}", candidates[0].0);
                    if prefix_unbalance.is_none() {
                        value = self.escape(&value);
                    }
                    return format!("{value}{space}");
                } else if let Some(common) = common_prefix(&values) {
                    if common != matcher && common != "--" {
                        let mut value = format!("{prefix}{common}");
                        if prefix_unbalance.is_none() {
                            value = self.escape(&value);
                        }
                        return value;
                    } else {
                        add_space_to_first_candidate = true;
                    }
                }
                candidates
                    .into_iter()
                    .enumerate()
                    .map(|(i, (value, description, nospace))| {
                        let mut new_value = self.escape(&value);
                        if i == 0 && add_space_to_first_candidate {
                            new_value = format!(" {}", new_value)
                        };
                        if nospace {
                            new_value
                        } else {
                            let description = self.comp_description(&description, "(", ")");
                            format!("{new_value} {description}")
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            Shell::Elvish | Shell::Powershell | Shell::Xonsh => candidates
                .into_iter()
                .map(|(value, description, nospace)| {
                    let new_value = self.comp_value1(prefix, &value, suffix);
                    let display = if value.is_empty() { " ".into() } else { value };
                    let description = self.comp_description(&description, "", "");
                    let space: &str = if nospace { "0" } else { "1" };
                    format!("{new_value}\t{space}\t{display}\t{description}")
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Fish => candidates
                .into_iter()
                .map(|(value, description, _nospace)| {
                    let new_value = self.comp_value1(prefix, &value, suffix);
                    let description = self.comp_description(&description, "\t", "");
                    format!("{new_value}{description}")
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Generic => candidates
                .into_iter()
                .map(|(value, description, nospace)| {
                    let description = self.comp_description(&description, "\t", "");
                    let space: &str = if nospace { "\0" } else { "" };
                    format!("{value}{space}{description}")
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Nushell => candidates
                .into_iter()
                .map(|(value, description, nospace)| {
                    let new_value = self.comp_value1(prefix, &value, suffix);
                    let space = if nospace { "" } else { " " };
                    let description = self.comp_description(&description, "\t", "");
                    format!("{new_value}{space}{description}")
                })
                .collect::<Vec<String>>()
                .join("\n"),
            Shell::Zsh => candidates
                .into_iter()
                .map(|(value, description, nospace)| {
                    let new_value = self.comp_value2(prefix, &value, suffix);
                    let display = value.replace(':', "\\:");
                    let description = self.comp_description(&description, ":", "");
                    let space = if nospace { "" } else { " " };
                    format!("{new_value}{space}\t{display}{description}")
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

    fn need_escape_chars(&self) -> &str {
        match self {
            Shell::Bash => r###"()<>"'` !#$&;\|"###,
            Shell::Nushell => r###"()[]{}"'` #$;|"###,
            Shell::Powershell => r###"()<>[]{}"'` #$&,;@|"###,
            Shell::Xonsh => r###"()<>[]{}!"'` #&:;\|"###,
            Shell::Zsh => r###"()<>[]"'` !#$&*:;?\|~"###,
            _ => "",
        }
    }

    fn need_break_chars(&self) -> Vec<char> {
        match self {
            Shell::Bash => match std::env::var("COMP_WORDBREAKS") {
                Ok(v) => v.chars().collect(),
                Err(_) => vec!['=', ':', '|', ';'],
            },
            Shell::Powershell => vec![','],
            _ => vec![],
        }
    }

    fn comp_value1(&self, prefix: &str, value: &str, suffix: &str) -> String {
        let mut new_value = format!("{prefix}{value}{suffix}");
        if unbalance_quote(prefix).is_none() {
            new_value = self.escape(&new_value);
        }
        new_value
    }

    fn comp_value2(&self, prefix: &str, value: &str, suffix: &str) -> String {
        let prefix = if let Some((_, i)) = unbalance_quote(prefix) {
            prefix
                .char_indices()
                .filter(|(v, _)| *v != i)
                .map(|(_, v)| v)
                .collect()
        } else {
            prefix.to_string()
        };
        self.escape(&format!("{prefix}{value}{suffix}"))
    }

    fn comp_description(&self, description: &str, prefix: &str, suffix: &str) -> String {
        if description.is_empty() || !self.with_description() {
            String::new()
        } else {
            format!("{prefix}{}{suffix}", truncate_description(description))
        }
    }
}

fn compgen_arg_value(value: &str) -> String {
    let value = value.to_lowercase();
    if ["path", "file", "arg", "any"]
        .iter()
        .any(|v| value.contains(v))
    {
        return "__argc_value:file".to_string();
    } else if value.contains("dir") || value.contains("folder") {
        return "__argc_value:dir".to_string();
    }
    String::new()
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
    let mut balances = vec![];
    for (i, c) in value.chars().enumerate() {
        if is_quote(c) {
            if balances.last().map(|(v, _)| v) == Some(&c) {
                balances.pop();
            } else {
                balances.push((c, i));
            }
        }
    }
    balances.first().cloned()
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

fn mod_quote(last: &mut String, prefix: &mut String, default_nospace: &mut bool) {
    if last.starts_with(is_quote) {
        prefix.push_str(&last[0..1]);
        *default_nospace = true;
    }
    *last = last.trim_matches(is_quote).to_string();
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
        assert_eq!(split_equal_sign("a="), Some(("a=", "")));
        assert_eq!(split_equal_sign("-a=c"), Some(("-a=", "c")));
        assert_eq!(split_equal_sign("a"), None);
        assert_eq!(split_equal_sign("a:"), None);
        assert_eq!(split_equal_sign("=a"), None);
    }
}
