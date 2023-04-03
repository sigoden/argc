mod completions;
mod utils;

use anyhow::{anyhow, bail, Context, Result};
use argc::{
    utils::{get_shell_path, no_color},
    Shell,
};
use clap::{Arg, ArgAction, Command};
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
    argc --argc-eval <SCRIPT> [ARGS...]           Use `eval "$(argc --argc-eval "$0" "$@")"`
    argc --argc-create [TASKS...]                 Create a boilerplate argcfile
    argc --argc-completions <SHELL> [CMDS...]     Generate completion scripts for bash,zsh,fish,powershell
    argc --argc-compgen <SHELL> <SCRIPT> <LINE>   Generate dynamic completion word
    argc --argc-export <SCRIPT>                   Export command line definitions as json
    argc --argc-script-path                       Print current argcfile path
    argc --argc-help                              Print help information
    argc --argc-version                           Print version information"#,
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
        let values = argc::eval(&source, &cmd_args)?;
        println!("{}", argc::ArgcValue::to_shell(values, no_color()))
    } else if matches.get_flag("argc-create") {
        if let Some((_, script_file)) = get_script_path(false) {
            bail!("Already exist {}", script_file.display());
        }
        let content = generate_boilerplate(&args[2..]);
        let names = candidate_script_names();
        fs::write(&names[0], content).with_context(|| format!("Failed to create {}", &names[0]))?;
        println!("{} has been successfully created.", &names[0]);
    } else if matches.get_flag("argc-export") {
        let (source, cmd_args) = parse_script_args(&args[2..])?;
        let json = argc::export(&source, &cmd_args[0])?;
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else if matches.get_flag("argc-compgen") {
        let shell: Shell = match args.get(2) {
            Some(v) => v.parse()?,
            None => bail!("Usage: argc --argc-compgen <SHELL> <SCRIPT> <LINE>"),
        };
        let (source, cmd_args) = parse_script_args(&args[3..])?;
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        let line = cmd_args.get(1).copied().unwrap_or_default();
        let output = argc::compgen(shell, &args[3], &source, line)?;
        println!("{output}");
    } else if matches.get_flag("argc-completions") {
        let shell: Shell = match args.get(2) {
            Some(v) => v.parse()?,
            None => bail!("Usage: argc --argc-completions <SHELL> [CMDS...]"),
        };
        let script = crate::completions::generate(shell, &args[3..])?;
        println!("{}", script);
    } else if matches.get_flag("argc-script-path") {
        let (_, script_file) =
            get_script_path(true).ok_or_else(|| anyhow!("Argcfile not found."))?;
        print!("{}", script_file.display());
    } else {
        let shell = get_shell_path().ok_or_else(|| anyhow!("Shell not found"))?;
        let (script_dir, script_file) = get_script_path(true)
            .ok_or_else(|| anyhow!("argcfile not found, try `argc --argc-help` for help."))?;
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
