use anyhow::{bail, Context, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const ARGC_SCRIPT_NAMES: [&str; 6] = [
    "Argcfile.sh",
    "Argcfile",
    "argcfile.sh",
    "argcfile",
    "ARGCFILE.sh",
    "ARGCFILE",
];

pub fn parse_script_args(args: &[String]) -> Result<(String, Vec<String>)> {
    if args.is_empty() {
        bail!("No script provided");
    }
    let script_file = args[0].as_str();
    let args: Vec<String> = args[1..].to_vec();
    let source = fs::read_to_string(script_file)
        .with_context(|| format!("Failed to load script at '{}'", script_file))?;
    let name = Path::new(script_file)
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap();
    let name = name.strip_suffix(".sh").unwrap_or(name);
    let mut cmd_args = vec![name.to_string()];
    cmd_args.extend(args);
    Ok((source, cmd_args))
}

pub fn generate_boilerplate(args: &[String]) -> String {
    let tasks = args
        .iter()
        .map(|cmd| {
            format!(
                r#"
# @cmd
{cmd}() {{
    echo To implement command: {cmd}
}}
"#
            )
        })
        .collect::<Vec<String>>()
        .join("");

    format!(
        r#"#!/usr/bin/env bash

set -e
{tasks}
eval "$(argc --argc-eval "$0" "$@")"
"#
    )
}

pub fn get_script_path(recursive: bool) -> Option<(PathBuf, PathBuf)> {
    let candidates = candidate_script_names();
    let mut dir = env::current_dir().ok()?;
    loop {
        for name in candidates.iter() {
            let path = dir.join(name);
            if path.exists() {
                return Some((dir, path));
            }
        }
        if !recursive {
            return None;
        }
        dir = dir.parent()?.to_path_buf();
    }
}

pub fn candidate_script_names() -> Vec<String> {
    let mut names = vec![];
    if let Ok(name) = env::var("ARGC_SCRIPT_NAME") {
        names.push(name.clone());
        if !name.ends_with(".sh") {
            names.push(format!("{name}.sh"));
        }
    }
    names.extend(ARGC_SCRIPT_NAMES.into_iter().map(|v| v.to_string()));
    names
}

pub fn search_completion_script(args: &mut Vec<String>) -> Option<PathBuf> {
    let search_paths = std::env::var("ARGC_COMPLETIONS_PATH").ok()?;
    let cmd = Path::new(&args[4])
        .file_stem()
        .and_then(|v| v.to_str())?
        .to_string();
    let subcmd = if args.len() >= 7
        && args[5]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
    {
        Some(args[5].clone())
    } else {
        None
    };
    let mut handlers = vec![];
    #[cfg(not(target_os = "windows"))]
    let sep = ":";
    #[cfg(target_os = "windows")]
    let sep = ";";
    for path in search_paths
        .split(sep)
        .filter(|v| !v.is_empty() && *v != sep)
    {
        let path = path.to_string();
        let cmd = cmd.to_string();
        let subcmd = subcmd.clone();
        let handler = std::thread::spawn(move || {
            if let Some(subcmd) = subcmd {
                let mut search_path = PathBuf::from(path.clone());
                search_path.push(cmd.clone());
                search_path.push(format!("{subcmd}.sh"));
                if search_path.exists() {
                    return Some((search_path, true));
                }
            }
            let mut search_path = PathBuf::from(path);
            search_path.push(format!("{cmd}.sh"));
            if search_path.exists() {
                return Some((search_path, false));
            }
            None
        });
        handlers.push(handler);
    }
    let mut subcmd_script_path = None;
    let mut cmd_script_path = None;
    for handler in handlers {
        if let Ok(Some((path, is_subcmd))) = handler.join() {
            if is_subcmd {
                subcmd_script_path = Some(path);
                break;
            } else if cmd_script_path.is_none() {
                cmd_script_path = Some(path);
            }
        }
    }
    if let (Some(path), Some(subcmd)) = (subcmd_script_path, subcmd) {
        args.remove(4);
        args[4] = format!("{cmd}-{subcmd}");
        return Some(path);
    }
    if let Some(path) = cmd_script_path {
        return Some(path);
    }
    None
}
