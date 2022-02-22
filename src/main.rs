use anyhow::{anyhow, Result};
use std::{env, fs, path::Path, process};

fn main() {
    let args: Vec<String> = env::args().into_iter().collect();
    let args = &args[..];
    if args.len() == 1 {
        show_help();
        process::exit(1);
    }
    let first = args[1].as_str();
    if first == "-h" || first == "--help" {
        show_help();
        process::exit(0);
    }
    let mut args = &args[1..];
    if first == "-e" || first == "--eval" {
        if args.len() == 1 {
            show_miss_file_error();
            process::exit(1);
        }
        args = &args[1..];
        match eval(&args) {
            Ok((stdout, stderr)) => {
                if let Some(stdout) = stdout {
                    print!("{}", &stdout)
                }
                if let Some(stderr) = stderr {
                    eprintln!("{}", &stderr)
                }
            }
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    } else {
        println!("{:?}", args);
    }
}

fn show_help() {
    println!(
        r###"{name} {version}

USAGE:
    {name} [OPTIONS] <FILE> [ARGS]...

ARGS:
    <FILE>          Specific the shell script file
    [ARGS]...       Arguments parss to shell script

OPTIONS:
    -e, --eval      Invoke in eval command
    -h, --help      Print help information
"###,
        name = env!("CARGO_CRATE_NAME"),
        version = env!("CARGO_PKG_VERSION")
    );
}

fn show_miss_file_error() {
    println!(
        r###"error: The following required arguments were not provided:
    <FILE>

USAGE:
    ${name} --eval <FILE>

For more information try --help"###,
        name = env!("CARGO_CRATE_NAME")
    );
}

fn eval(args: &[String]) -> Result<(Option<String>, Option<String>)> {
    let script_file = args[0].as_str();
    let args: Vec<&str> = args[1..].iter().map(|v| v.as_str()).collect();
    eprintln!("{} {:?}", script_file, args);
    let name = Path::new(script_file)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or(env!("CARGO_CRATE_NAME"));
    let source =
        fs::read_to_string(script_file).map_err(|e| anyhow!("Fail to read script, {}", e))?;
    let mut cmd_args = vec![name];
    cmd_args.extend(args);
    argc::run(&source, &cmd_args)
}
