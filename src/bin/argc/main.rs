mod completions;
mod utils;

use anyhow::{anyhow, bail, Context, Result};
use argc::{
    utils::{get_shell_path, termwidth},
    Shell,
};
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
    let mut argc_cmd = None;
    if let Some(arg) = args.get(1) {
        if arg.starts_with("--argc-") {
            argc_cmd = Some(arg.as_str());
        }
    };

    if let Some(argc_cmd) = argc_cmd {
        match argc_cmd {
            "--argc-eval" => {
                let (source, cmd_args) = parse_script_args(&args[2..])?;
                let values = argc::eval(Some(&args[2]), &source, &cmd_args, termwidth())?;
                println!("{}", argc::ArgcValue::to_shell(values))
            }
            "--argc-create" => {
                if let Some((_, script_file)) = get_script_path(false) {
                    bail!("Already exist {}", script_file.display());
                }
                let content = generate_boilerplate(&args[2..]);
                let names = candidate_script_names();
                fs::write(&names[0], content)
                    .with_context(|| format!("Failed to create {}", &names[0]))?;
                println!("{} has been successfully created.", &names[0]);
            }
            "--argc-export" => {
                let (source, _) = parse_script_args(&args[2..])?;
                let json = argc::export(&source)?;
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
            "--argc-compgen" => {
                let shell: Shell = match args.get(2) {
                    Some(v) => v.parse()?,
                    None => bail!("Usage: argc --argc-compgen <SHELL> <SCRIPT> <LINE>"),
                };
                let (source, cmd_args) = parse_script_args(&args[3..])?;
                let line = cmd_args.get(1).cloned().unwrap_or_default();
                let output = argc::compgen(shell, &args[3], &source, &cmd_args[0], &line)?;
                println!("{output}");
            }
            "--argc-completions" => {
                let shell: Shell = match args.get(2) {
                    Some(v) => v.parse()?,
                    None => bail!("Usage: argc --argc-completions <SHELL> [CMDS...]"),
                };
                let script = crate::completions::generate(shell, &args[3..])?;
                println!("{}", script);
            }
            "--argc-script-path" => {
                let (_, script_file) =
                    get_script_path(true).ok_or_else(|| anyhow!("Argcfile not found."))?;
                print!("{}", script_file.display());
            }
            "--argc-help" => {
                println!("{}", get_argc_help())
            }
            "--argc-version" => {
                println!("{}", get_argc_version())
            }
            _ => {
                bail!("Invalid option `{argc_cmd}`")
            }
        }
        Ok(0)
    } else {
        let shell = get_shell_path().ok_or_else(|| anyhow!("Shell not found"))?;
        let (script_dir, script_file) = get_script_path(true)
            .ok_or_else(|| anyhow!("Argcfile not found, try `argc --argc-help` for help."))?;
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
            Ok(130)
        } else {
            Ok(status.code().unwrap_or_default())
        }
    }
}

fn get_argc_help() -> String {
    let name = env!("CARGO_CRATE_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let about = concat!(
        env!("CARGO_PKG_DESCRIPTION"),
        " - ",
        env!("CARGO_PKG_REPOSITORY")
    );
    format!(
        r###"
{name} {version}
{about}

USAGE:
    argc --argc-eval <SCRIPT> [ARGS...]           Use `eval "$(argc --argc-eval "$0" "$@")"`
    argc --argc-create [TASKS...]                 Create a boilerplate argcfile
    argc --argc-completions <SHELL> [CMDS...]     Generate completion scripts for bash,zsh,fish,powershell
    argc --argc-compgen <SHELL> <SCRIPT> <LINE>   Generate dynamic completion word
    argc --argc-export <SCRIPT>                   Export command line definitions as json
    argc --argc-script-path                       Print current argcfile path
    argc --argc-help                              Print help information
    argc --argc-version                           Print version information
"###
    )
}

fn get_argc_version() -> String {
    let name = env!("CARGO_CRATE_NAME");
    let version = env!("CARGO_PKG_VERSION");
    format!("{name} {version}\n")
}
