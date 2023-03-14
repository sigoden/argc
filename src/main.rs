use anyhow::{anyhow, bail, Result};
use clap::{Arg, ArgAction, Command};
use either::Either;
use std::{env, fs, path::Path, process};

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
    let mut argc_args: Vec<String> = if args.len() == 1 {
        vec![args[0].clone(), "-h".to_owned()]
    } else {
        vec![args[0].clone()]
    };
    let mut script_args: Vec<String> = vec![];
    for arg in args.into_iter().skip(1) {
        if script_args.is_empty() {
            if arg.trim().starts_with('-') {
                argc_args.push(arg);
                continue;
            }
            script_args.push(arg);
        } else {
            script_args.push(arg);
        }
    }

    let matches = Command::new(env!("CARGO_CRATE_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .override_usage(
            r#"
    argc SCRIPT [ARGS...]               Parse arguments `eval "$(argc "$0" "$@")"`
    argc --help                         Print help information
    argc --version                      Print version information"#,
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
        .arg(
            Arg::new("compgen")
                .long("compgen")
                .action(ArgAction::SetTrue),
        )
        .try_get_matches_from(&argc_args)?;

    if matches.get_flag("compgen") {
        let (source, cmd_args) = parse_script_args(&script_args)?;
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        let line = if cmd_args.len() == 1 { "" } else { cmd_args[1] };
        let candicates = argc::compgen(&source, line)?;
        candicates.into_iter().for_each(|v| println!("{v}"));
    } else {
        let (source, cmd_args) = parse_script_args(&script_args)?;
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
