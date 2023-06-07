use convert_case::{Boundary, Converter, Pattern};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process, thread,
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

pub fn is_choice_value_terminate(c: char) -> bool {
    c == '|' || c == ']'
}

pub fn is_default_value_terminate(c: char) -> bool {
    c.is_whitespace()
}

pub fn get_shell_path() -> Option<PathBuf> {
    let shell = match env::var("ARGC_SHELL_PATH") {
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
    let git = which("git").ok()?;
    Some(git.parent()?.parent()?.join("bin").join("bash.exe"))
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
) -> Option<Vec<Vec<String>>> {
    let shell = get_shell_path()?;
    let shell_extra_args = if shell
        .file_stem()
        .map(|v| v.to_string_lossy().to_lowercase().contains("bash"))
        .unwrap_or_default()
    {
        vec!["--noprofile".to_string(), "--norc".to_string()]
    } else {
        vec![]
    };
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
    let list: Vec<Vec<String>> = handles
        .into_iter()
        .map(|h| h.join().ok().unwrap_or_default())
        .map(|v| {
            v.split('\n')
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
                .collect()
        })
        .collect();
    Some(list)
}

pub fn termwidth() -> Option<usize> {
    env::var("TERM_WIDTH").ok()?.parse().ok()
}

pub fn get_current_dir() -> Option<String> {
    env::current_dir()
        .ok()
        .map(|v| v.to_string_lossy().to_string())
}

fn path_env_with_exe() -> String {
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
