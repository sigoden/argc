use anyhow::{anyhow, bail, Result};
use clap::{Arg, ArgAction, Command};
use either::Either;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};
use which::which;

fn main() {
    match run() {
        Ok(code) => {
            if code != 0 {
                process::exit(code);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

fn run() -> Result<i32> {
    let mut argc_args: Vec<String> = vec![];
    let mut script_args: Vec<String> = vec![];
    let mut next_argc_arg = true;
    for arg in std::env::args() {
        if next_argc_arg {
            argc_args.push(arg);
            next_argc_arg = false;
            continue;
        }
        if script_args.is_empty() && arg.starts_with("--argc-") {
            argc_args.push(arg);
            continue;
        }
        next_argc_arg = false;
        script_args.push(arg);
    }
    let matches = Command::new(env!("CARGO_CRATE_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .override_usage(
            r#"
    argc --argc-help                               Print help information
    argc --argc-eval SCRIPT [ARGS ...]             Parse arguments `eval $(argc --argc-eval "$0" "$@")`
    argc --argc-create [TASKS ...]                 Create a boilerplate argcfile
    argc --argc-compgen SCRIPT [ARGS ...]          Print commands and options as completion candidates 
    argc --argc-argcfile                           Print argcfile path
    argc --argc-version                            Print version information"#,
        )
        .help_template(r#"{bin} {version}
{author}
{about}

USAGE:{usage}"#)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .about(concat!(
            env!("CARGO_PKG_DESCRIPTION"),
            " - ",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .arg(Arg::new("argc-eval").long("argc-eval").action(ArgAction::SetTrue))
        .arg(Arg::new("argc-create").long("argc-create").action(ArgAction::SetTrue))
        .arg(
            Arg::new("argc-compgen")
                .long("argc-compgen").action(ArgAction::SetTrue))
        .arg(
            Arg::new("argc-argcfile")
                .long("argc-argcfile").action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("argc-version")
                .long("argc-version")
                .action(ArgAction::Version)
        )
        .arg(
            Arg::new("argc-help")
                .long("argc-help")
                .action(ArgAction::Help)
        )
        .try_get_matches_from(&argc_args)?;

    if matches.get_flag("argc-eval") {
        let (source, cmd_args) = parse_script_args(&script_args)?;
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        match argc::run(&source, &cmd_args)? {
            Either::Left(stdout) => {
                println!("{}", stdout)
            }
            Either::Right(stderr) => {
                eprintln!("{}", stderr);
                println!("exit 1");
            }
        }
    } else if matches.get_flag("argc-create") {
        if let Some((_, script_file)) = get_script_path(false) {
            bail!("Already exist {}", script_file.display());
        }
        let content = generate_boilerplate(&script_args);
        let names = candidate_script_names();
        fs::write(&names[0], content)
            .map_err(|err| anyhow!("Failed to create argcfile.sh, {err}"))?;
    } else if matches.get_flag("argc-argcfile") {
        let (_, script_file) =
            get_script_path(true).ok_or_else(|| anyhow!("Not found script file"))?;
        print!("{}", script_file.display());
    } else if matches.get_flag("argc-compgen") {
        let (source, cmd_args) = parse_script_args(&script_args)?;
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        print!("{}", argc::compgen(&source, &cmd_args)?.join(" "))
    } else {
        let shell = get_shell_path().ok_or_else(|| anyhow!("Not found shell"))?;
        let (script_dir, script_file) =
            get_script_path(true).ok_or_else(|| anyhow!("Not found script file"))?;
        let mut command = process::Command::new(&shell);
        command.arg(&script_file);
        command.args(&script_args);
        command.current_dir(script_dir);
        let status = command
            .status()
            .map_err(|err| anyhow!("Run `{}` throw {}", script_file.display(), err))?;
        return Ok(status.code().unwrap_or_default());
    }
    Ok(0)
}

fn parse_script_args(args: &[String]) -> Result<(String, Vec<String>)> {
    if args.is_empty() {
        bail!("No script file");
    }
    let script_file = args[0].as_str();
    let args: Vec<String> = args[1..].to_vec();
    let source = fs::read_to_string(script_file)
        .map_err(|e| anyhow!("Failed to load '{}', {}", script_file, e))?;
    let name = Path::new(script_file)
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| anyhow!("Failed to get script name"))?;
    let mut cmd_args = vec![name.to_string()];
    cmd_args.extend(args);
    Ok((source, cmd_args))
}

fn generate_boilerplate(args: &[String]) -> String {
    let tasks = args
        .iter()
        .map(|cmd| {
            format!(
                r#"
# @cmd
{cmd}() {{
    echo Run {cmd}
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
eval $(argc --argc-eval "$0" "$@")
"#
    )
}

fn get_script_path(recursive: bool) -> Option<(PathBuf, PathBuf)> {
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

fn candidate_script_names() -> Vec<String> {
    let mut names = vec![];
    if let Ok(name) = env::var("ARGC_SCRIPT") {
        names.push(name.clone());
        if !name.ends_with(".sh") {
            names.push(format!("{}.sh", name));
        }
    }
    names.push("argcfile.sh".into());
    names.push("Argcfile.sh".into());
    names.push("argcfile".into());
    names.push("Argcfile".into());
    names
}

fn get_shell_path() -> Option<PathBuf> {
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
fn get_bash_path() -> Option<PathBuf> {
    if let Ok(bash) = which("bash") {
        if bash.display().to_string().to_lowercase() != "c:\\windows\\system32\\bash.exe" {
            return Some(bash);
        }
    }
    let git = which("git").ok()?;
    Some(git.parent()?.parent()?.join("bin").join("bash.exe"))
}

#[cfg(not(windows))]
fn get_bash_path() -> Option<PathBuf> {
    which("bash").ok()
}
