use anyhow::{anyhow, Result};
use std::{env, fs, path::Path, process};

fn main() {
    let args: Vec<String> = env::args().into_iter().collect();
    if args.len() == 1 {
        show_help();
        process::exit(1);
    }
    match args[1].as_str() {
        "-h" | "--help" => {
            show_help();
        }
        "-V" | "--version" => {
            println!("{} {}", env!("CARGO_CRATE_NAME"), env!("CARGO_PKG_VERSION"));
        }
        _ => match run(&args[1..]) {
            Ok((stdout, stderr)) => {
                if let Some(stdout) = stdout {
                    print!("{}", &stdout)
                }
                if let Some(stderr) = stderr {
                    eprintln!("{}", &stderr)
                }
            }
            Err(err) => {
                eprintln!("error: {}", err);
                std::process::exit(1);
            }
        },
    }
}

fn show_help() {
    println!(
        r###"{name} {version}
{description} - {repository}

USAGE:
    {name} [OPTIONS] <FILE> [ARGS]...

ARGS:
    <FILE>          Specific the shell script file
    [ARGS]...       Arguments parss to shell script

OPTIONS:
    -h, --help      Print help information
    -V, --version   Print version information
"###,
        name = env!("CARGO_CRATE_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        description = env!("CARGO_PKG_DESCRIPTION"),
        repository = env!("CARGO_PKG_REPOSITORY")
    );
}

fn run(args: &[String]) -> Result<(Option<String>, Option<String>)> {
    let script_file = args[0].as_str();
    let args: Vec<&str> = args[1..].iter().map(|v| v.as_str()).collect();
    let name = Path::new(script_file)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or(env!("CARGO_CRATE_NAME"));
    let source = fs::read_to_string(script_file)
        .map_err(|e| anyhow!("Fail to load '{}', {}", script_file, e))?;
    let mut cmd_args = vec![name];
    cmd_args.extend(args);
    argc::run(&source, &cmd_args)
}
