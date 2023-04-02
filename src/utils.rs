use anyhow::{anyhow, Context, Result};
use convert_case::{Boundary, Converter, Pattern};
use std::{
    env,
    path::{Path, PathBuf},
};
use which::which;

/// Transform into upper case string with an underscore between words. `foo-bar` => `FOO-BAR`
pub fn to_cobol_case(value: &str) -> String {
    Converter::new()
        .set_pattern(Pattern::Uppercase)
        .set_delim("-")
        .set_boundaries(&[Boundary::Underscore, Boundary::LowerUpper, Boundary::Hyphen])
        .convert(value)
}

pub fn hyphens_to_underscores(name: &str) -> String {
    name.replace('-', "_")
}

pub fn escape_shell_words(value: &str) -> String {
    shell_words::quote(value).to_string()
}

pub fn split_shell_words(s: &str) -> Result<Vec<String>> {
    shell_words::split(s).with_context(|| anyhow!("Failed to split shell words"))
}

pub fn is_choice_value_terminate(c: char) -> bool {
    c == '|' || c == ']'
}

pub fn is_default_value_terminate(c: char) -> bool {
    c.is_whitespace()
}

pub fn get_shell_path() -> Option<PathBuf> {
    let shell = match env::var("ARGC_SHELL") {
        Ok(v) => Path::new(&v).to_path_buf(),
        Err(_) => get_bash_path()?,
    };
    if !shell.exists() {
        return None;
    }
    Some(shell)
}

#[cfg(windows)]
pub fn get_bash_path() -> Option<PathBuf> {
    let git_bash_path = PathBuf::from("C:\\Program Files\\Git\\bin\\bash.exe");
    if git_bash_path.exists() {
        return Some(git_bash_path);
    }
    if let Ok(bash) = which("bash") {
        if bash.display().to_string().to_lowercase() != "c:\\windows\\system32\\bash.exe" {
            return Some(bash);
        }
    }
    let git = which("git").ok()?;
    Some(git.parent()?.parent()?.join("bin").join("bash.exe"))
}

#[cfg(not(windows))]
pub fn get_bash_path() -> Option<PathBuf> {
    which("bash").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cobol() {
        assert_eq!("FOO-BAR".to_string(), to_cobol_case("fooBar"));
        assert_eq!("FOO-BAR".to_string(), to_cobol_case("foo-bar"));
        assert_eq!("FOO1".to_string(), to_cobol_case("foo1"));
    }
}
