use anyhow::{anyhow, bail, Context, Result};
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
        bail!("No script file");
    }
    let script_file = args[0].as_str();
    let args: Vec<String> = args[1..].to_vec();
    let source = fs::read_to_string(script_file)
        .with_context(|| format!("Failed to load '{}'", script_file))?;
    let name = Path::new(script_file)
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| anyhow!("Failed to get script name"))?;
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
