mod completions;

use anyhow::{anyhow, bail, Result};
use clap::{Arg, ArgAction, Command};
use either::Either;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use which::which;

const ARGC_SCRIPT_NAMES: [&str; 6] = [
    "Argcfile.sh",
    "Argcfile",
    "argcfile.sh",
    "argcfile",
    "ARGCFILE.sh",
    "ARGCFILE",
];

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
    let args: Vec<String> = std::env::args().collect();
    let mut argc_args = vec![args[0].clone()];
    if let Some(arg) = args.get(1) {
        if arg.starts_with("--argc-") {
            argc_args.push(args[1].clone());
        }
    }
    let matches = Command::new(env!("CARGO_CRATE_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .override_usage(
            r#"
    argc --argc-eval SCRIPT [ARGS...]           Parse arguments `eval "$(argc --argc-eval "$0" "$@")"`
    argc --argc-compgen SCRIPT LINE             Generate words for completion
    argc --argc-create [TASKS...]               Create a boilerplate argcfile
    argc --argc-export SCRIPT                   Export command definitions as json
    argc --argc-script-path                     Print current argcscript file path
    argc --argc-completions SHELL [CMDS...]     Generate completion scripts for bash,zsh,fish,powershell
    argc --argc-help                            Print help information
    argc --argc-version                         Print version information"#,
        )
        .help_template(
            r#"{bin} {version}
{author}
{about}

USAGE:{usage}"#,
        )
        .about(concat!(
            env!("CARGO_PKG_DESCRIPTION"),
            " - ",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .arg(Arg::new("argc-eval").long("argc-eval").action(ArgAction::SetTrue))
        .arg(Arg::new("argc-create").long("argc-create").action(ArgAction::SetTrue))
        .arg(Arg::new("argc-export").long("argc-export").action(ArgAction::SetTrue))
        .arg(Arg::new("argc-compgen").long("argc-compgen").action(ArgAction::SetTrue))
        .arg(Arg::new("argc-completions").long("argc-completions").action(ArgAction::SetTrue))
        .arg(Arg::new("argc-script-path").long("argc-script-path").action(ArgAction::SetTrue))
        .arg(Arg::new("argc-help").long("argc-help").action(ArgAction::Help))
        .arg(Arg::new("argc-version").long("argc-version").action(ArgAction::Version))
        .try_get_matches_from(&argc_args)?;

    if matches.get_flag("argc-eval") {
        let (source, cmd_args) = parse_script_args(&args[2..])?;
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        match argc::eval(&source, &cmd_args)? {
            Either::Left(output) => {
                println!("{}", output)
            }
            Either::Right(error) => {
                if env::var_os("NO_COLOR").is_some() {
                    eprintln!("{}", error);
                } else {
                    eprintln!("{}", error.render().ansi());
                }
                if error.use_stderr() {
                    println!("exit 1");
                } else {
                    println!("exit 0");
                }
            }
        }
    } else if matches.get_flag("argc-create") {
        if let Some((_, script_file)) = get_script_path(false) {
            bail!("Already exist {}", script_file.display());
        }
        let content = generate_boilerplate(&args[2..]);
        let names = candidate_script_names();
        fs::write(&names[0], content).map_err(|err| anyhow!("Failed to create argc.sh, {err}"))?;
    } else if matches.get_flag("argc-export") {
        let (source, _) = parse_script_args(&args[2..])?;
        let json = argc::export(&source)?;
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else if matches.get_flag("argc-compgen") {
        let (source, cmd_args) = parse_script_args(&args[2..])?;
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        let line = if cmd_args.len() == 1 { "" } else { cmd_args[1] };
        let candicates = argc::compgen(&source, line)?;
        let candicates = expand_candicates(candicates, &args[2])?;
        candicates.into_iter().for_each(|v| println!("{v}"));
    } else if matches.get_flag("argc-completions") {
        let script = crate::completions::generate(&args[2..])?;
        println!("{}", script);
    } else if matches.get_flag("argc-script-path") {
        let (_, script_file) =
            get_script_path(true).ok_or_else(|| anyhow!("Not found argcfile"))?;
        print!("{}", script_file.display());
    } else {
        let shell = get_shell_path().ok_or_else(|| anyhow!("Not found shell"))?;
        let (script_dir, script_file) = get_script_path(true)
            .ok_or_else(|| anyhow!("Not found argcscript, try `argc --argc-help` to get help."))?;
        let interrupt = Arc::new(AtomicBool::new(false));
        let interrupt_me = interrupt.clone();
        ctrlc::set_handler(move || interrupt_me.store(true, Ordering::Relaxed))
            .map_err(|err| anyhow!("Failed to set CTRL-C handler: {}", err))?;
        let mut command = process::Command::new(shell);
        command.arg(&script_file);
        command.args(&args[1..]);
        command.current_dir(script_dir);
        let status = command
            .status()
            .map_err(|err| anyhow!("Run `{}` throw {}", script_file.display(), err))?;
        if interrupt.load(Ordering::Relaxed) {
            return Ok(130);
        }
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

fn expand_candicates(values: Vec<String>, script_file: &str) -> Result<Vec<String>> {
    let mut output = vec![];
    let mut param_fns = vec![];
    for value in values {
        let value_len = value.len();
        if value_len > 2 && value.starts_with('`') && value.ends_with('`') {
            param_fns.push(value.as_str()[1..value_len - 1].to_string());
        } else {
            output.push(value);
        }
    }
    if !param_fns.is_empty() {
        if let Some(shell) = get_shell_path() {
            for param_fn in param_fns {
                if let Ok(fn_output) = process::Command::new(&shell)
                    .arg(script_file)
                    .arg(&param_fn)
                    .output()
                {
                    let fn_output = String::from_utf8_lossy(&fn_output.stdout);
                    for line in fn_output.split('\n') {
                        let line = line.trim();
                        if !line.is_empty() {
                            output.push(line.to_string());
                        }
                    }
                }
            }
        }
    }
    if output.len() == 1 {
        let candicate = output[0].to_ascii_uppercase();
        let is_arg_value = (candicate.starts_with('<')
            && (candicate.ends_with('>') || candicate.ends_with(">...")))
            || (candicate.starts_with('[')
                && (candicate.ends_with(']') || candicate.ends_with("]...")));
        if is_arg_value {
            if candicate.contains("PATH") || candicate.contains("FILE") {
                output[0] = "__argc_comp:file".into();
            } else if candicate.contains("DIR") || candicate.contains("FOLDER") {
                output[0] = "__argc_comp:dir".into();
            } else {
                output.clear();
            }
        }
    }
    Ok(output)
}

fn generate_boilerplate(args: &[String]) -> String {
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
    if let Ok(name) = env::var("ARGC_SCRIPT_NAME") {
        names.push(name.clone());
        if !name.ends_with(".sh") {
            names.push(format!("{name}.sh"));
        }
    }
    names.extend(ARGC_SCRIPT_NAMES.into_iter().map(|v| v.to_string()));
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
fn get_bash_path() -> Option<PathBuf> {
    which("bash").ok()
}
