use anyhow::{anyhow, Result};
use clap::{arg, Command, ErrorKind};
use std::{env, fs, path::Path, process};

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
        .arg(arg!(-e --eval "Adjust to run in sh eval"))
        .arg(arg!(<SCRIPT> "Script file to be parsed"))
        .arg(arg!([ARGUMENTS]... "Arguments passed to script file"))
        .try_get_matches_from(&args);

    match res {
        Ok(matches) => {
            let eval = matches.is_present("eval");
            match run(&script_args, eval) {
                Ok(result) => match result {
                    Ok(stdout) => {
                        println!("{}", stdout)
                    }
                    Err(stderr) => {
                        eprintln!("{}", stderr);
                        if eval {
                            println!("exit 1");
                        } else {
                            process::exit(1);
                        }
                    }
                },
                Err(err) => {
                    eprintln!("error: {}", err);
                    process::exit(1);
                }
            }
        }
        Err(err) => {
            if err.kind() == ErrorKind::DisplayHelp {
                println!("{}", err);
            } else {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
    }
}
fn run(args: &[String], eval: bool) -> Result<std::result::Result<String, String>> {
    let script_file = args[0].as_str();
    let args: Vec<&str> = args[1..].iter().map(|v| v.as_str()).collect();
    let name = Path::new(script_file)
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or_else(|| anyhow!("Fail to get command name"))?;
    let source = fs::read_to_string(script_file)
        .map_err(|e| anyhow!("Fail to load '{}', {}", script_file, e))?;
    let mut cmd_args = vec![name];
    cmd_args.extend(args);
    let runner = argc::Runner::new(&source).set_eval(eval);
    runner.run(&cmd_args)
}
