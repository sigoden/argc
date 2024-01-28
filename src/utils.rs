use convert_case::{Boundary, Converter, Pattern};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process, thread,
};
use which::which;

pub const INTERNAL_SYMBOL: &str = "___internal___";

pub fn to_cobol_case(value: &str) -> String {
    Converter::new()
        .set_pattern(Pattern::Uppercase)
        .set_delim("-")
        .set_boundaries(&[Boundary::Underscore, Boundary::LowerUpper, Boundary::Hyphen])
        .convert(value)
}

pub fn escape_shell_words(value: &str) -> String {
    shell_words::quote(value).to_string()
}

pub fn is_choice_value_terminate(c: char) -> bool {
    c == '|' || c == ']'
}

pub fn is_quote_char(c: char) -> bool {
    c == '\'' || c == '"'
}

pub fn is_default_value_terminate(c: char) -> bool {
    c.is_whitespace()
}

pub fn get_shell_path() -> anyhow::Result<PathBuf> {
    match env::var("ARGC_SHELL_PATH") {
        Ok(v) => {
            let shell_path = Path::new(&v).to_path_buf();
            if !shell_path.exists() {
                anyhow::bail!(
                    "Invalid ARGC_SHELL_PATH, '{}' does not exist",
                    shell_path.display()
                );
            }
            Ok(shell_path)
        }
        Err(_) => get_bash_path().ok_or_else(|| anyhow::anyhow!("Shell not found")),
    }
}

pub fn get_shell_args(shell_path: &Path) -> Vec<String> {
    if let Some(name) = shell_path
        .file_stem()
        .map(|v| v.to_string_lossy().to_lowercase())
    {
        match name.as_str() {
            "bash" => vec!["--noprofile".to_string(), "--norc".to_string()],
            _ => vec![],
        }
    } else {
        vec![]
    }
}

#[cfg(windows)]
pub fn get_bash_path() -> Option<PathBuf> {
    let bash_path = PathBuf::from("C:\\Program Files\\Git\\bin\\bash.exe");
    if bash_path.exists() {
        return Some(bash_path);
    }
    let git = which("git").ok()?;
    let parent = git.parent()?;
    let bash_path = parent.parent()?.join("bin").join("bash.exe");
    if bash_path.exists() {
        return Some(bash_path);
    }
    let bash_path = parent.join("bash.exe");
    if bash_path.exists() {
        return Some(bash_path);
    }
    None
}

#[cfg(not(windows))]
pub fn get_bash_path() -> Option<PathBuf> {
    which("bash").ok()
}

pub fn run_param_fns(
    script_file: &str,
    param_fns: &[&str],
    args: &[String],
    envs: HashMap<String, String>,
) -> Option<Vec<String>> {
    let shell = get_shell_path().ok()?;
    let shell_extra_args = get_shell_args(&shell);
    let path_env = path_env_with_exe();
    let handles: Vec<_> = param_fns
        .iter()
        .map(|param_fn| {
            let script_file = script_file.to_string();
            let args: Vec<String> = args.to_vec();
            let path_env = path_env.clone();
            let param_fn = param_fn.to_string();
            let shell = shell.clone();
            let shell_extra_args = shell_extra_args.clone();
            let envs = envs.clone();
            thread::spawn(move || {
                process::Command::new(shell)
                    .args(shell_extra_args)
                    .arg(&script_file)
                    .arg(INTERNAL_SYMBOL)
                    .arg(&param_fn)
                    .args(args)
                    .envs(envs)
                    .env("PATH", path_env)
                    .output()
                    .ok()
                    .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
                    .unwrap_or_default()
            })
        })
        .collect();
    let result: Vec<String> = handles
        .into_iter()
        .map(|h| {
            h.join()
                .ok()
                .map(|v| v.trim().to_string())
                .unwrap_or_default()
        })
        .collect();
    Some(result)
}

pub fn termwidth() -> Option<usize> {
    env::var("TERM_WIDTH").ok()?.parse().ok()
}

pub fn is_windows_path(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    ('a'..='z').any(|v| {
        if value.len() == 2 {
            value == format!("{v}:")
        } else {
            value.starts_with(&format!("{v}:/"))
        }
    })
}

pub fn get_current_dir() -> Option<String> {
    env::current_dir()
        .ok()
        .map(|v| v.to_string_lossy().to_string())
}

pub fn path_env_with_exe() -> String {
    let mut path_env = std::env::var("PATH").ok().unwrap_or_default();
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|v| v.parent().map(|v| v.to_path_buf()))
    {
        if cfg!(windows) {
            path_env = format!("{};{}", exe_dir.display(), path_env)
        } else {
            path_env = format!("{}:{}", exe_dir.display(), path_env)
        }
    }
    path_env
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
