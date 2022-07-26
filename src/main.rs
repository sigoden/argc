use anyhow::{anyhow, Result};
use clap::{arg, ArgMatches, Command, ErrorKind};
use std::{env, fs, io, path::Path, process};

fn main() {
    let mut args: Vec<String> = vec![];
    let mut script_args: Vec<String> = vec![];
    for arg in std::env::args() {
        if script_args.is_empty() {
            if !args.is_empty() && !arg.trim().starts_with('-') {
                script_args.push(arg.clone());
            }
            args.push(arg)
        } else {
            script_args.push(arg);
        }
    }
    let res = Command::new(env!("CARGO_CRATE_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .disable_help_subcommand(true)
        .about(concat!(
            env!("CARGO_PKG_DESCRIPTION"),
            " - ",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .arg(arg!(-e --eval "Adjust to run in sh eval").hide(true)) // will be removed in v1.0
        .arg(arg!(-c --completion "Print bash completion script"))
        .arg(arg!(<SCRIPT> "Script file to be parsed"))
        .arg(arg!([ARGUMENTS]... "Arguments passed to script file"))
        .try_get_matches_from(&args);

    if let Err(err) = res.map(|v| start(v, &script_args)) {
        if err.kind() == ErrorKind::DisplayHelp {
            println!("{}", err);
        } else {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

fn start(matches: ArgMatches, script_args: &[String]) -> Result<()> {
    let (source, cmd_args) = proc_script_args(script_args)?;
    let runner = argc::Runner::new(&source);
    let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
    if matches.is_present("completion") {
        runner.complete(cmd_args[0], &mut io::stdout())?;
    } else {
        match runner.run(&cmd_args)? {
            Ok(stdout) => {
                println!("{}", stdout)
            }
            Err(stderr) => {
                eprintln!("{}", stderr);
                println!("exit 1");
            }
        }
    }
    Ok(())
}

fn proc_script_args(args: &[String]) -> Result<(String, Vec<String>)> {
    let script_file = args[0].as_str();
    let args: Vec<String> = args[1..].to_vec();
    let name = Path::new(script_file)
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or_else(|| anyhow!("Fail to get command name"))?;
    let source = fs::read_to_string(script_file)
        .map_err(|e| anyhow!("Fail to load '{}', {}", script_file, e))?;
    let mut cmd_args = vec![name.to_string()];
    cmd_args.extend(args);
    Ok((source, cmd_args))
}
