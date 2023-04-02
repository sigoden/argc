mod compgen;
mod completions;
mod utils;

use crate::compgen::Shell;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Arg, ArgAction, Command};
use either::Either;
use std::{
    env, fs, process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use utils::*;

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
    argc --argc-compgen SHELL SCRIPT LINE       Generate possible completions for the shell
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
        fs::write(&names[0], content).with_context(|| format!("Failed to create {}", &names[0]))?;
        println!("{} has been successfully created.", &names[0]);
    } else if matches.get_flag("argc-export") {
        let (source, _) = parse_script_args(&args[2..])?;
        let json = argc::export(&source)?;
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else if matches.get_flag("argc-compgen") {
        let shell: Shell = match args.get(2) {
            Some(v) => v.parse()?,
            None => bail!(
                "No shell specified, Please specify the one of {}",
                Shell::list()
            ),
        };
        let (source, cmd_args) = parse_script_args(&args[3..])?;
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        let output = crate::compgen::generate(shell, &args[3], &source, &cmd_args)?;
        println!("{output}");
    } else if matches.get_flag("argc-completions") {
        let shell: Shell = match args.get(2) {
            Some(v) => v.parse()?,
            None => bail!(
                "No shell specified, Please specify the one of {}",
                Shell::list()
            ),
        };
        let script = crate::completions::generate(shell, &args[3..])?;
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
            .with_context(|| "Failed to set CTRL-C handler")?;
        let mut command = process::Command::new(shell);
        command.arg(&script_file);
        command.args(&args[1..]);
        command.current_dir(script_dir);
        let status = command
            .status()
            .with_context(|| format!("Failed to run `{}`", script_file.display()))?;
        if interrupt.load(Ordering::Relaxed) {
            return Ok(130);
        }
        return Ok(status.code().unwrap_or_default());
    }

    Ok(0)
}
