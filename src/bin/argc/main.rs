mod completions;
mod parallel;
mod utils;

use anyhow::{anyhow, bail, Context, Result};
use argc::{
    utils::{escape_shell_words, get_current_dir, get_shell_path, termwidth},
    Shell,
};
use base64::{engine::general_purpose, Engine as _};
use std::{collections::HashMap, env, fs, path::Path, process};
use utils::*;
use which::which;

const COMPLETION_SCRIPT: &str = include_str!("completions/completion.sh");

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
                if cmd_args
                    .get(1)
                    .map(|v| v == parallel::PARALLEL_MODE)
                    .unwrap_or_default()
                {
                    let cmd_args_len = cmd_args.len();
                    if cmd_args_len < 3 {
                        bail!("No parallel command")
                    }
                    let mut code = retrive_argc_variables().unwrap_or_default();
                    let mut cmds = vec![cmd_args[2].to_string()];
                    cmds.extend(cmd_args[3..].iter().map(|v| escape_shell_words(v)));
                    code.push_str(&cmds.join(" "));
                    println!("{code}")
                } else {
                    let values = argc::eval(&source, &cmd_args, Some(&args[2]), termwidth())?;
                    let dir_vars = export_dir_vars(&args[2]);
                    let code = argc::ArgcValue::to_shell(&values);
                    let export_vars = export_argc_variables(&code);
                    println!("{dir_vars}{export_vars}{code}")
                }
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
                run_compgen(args.to_vec());
            }
            "--argc-completions" => {
                let shell: Shell = match args.get(2) {
                    Some(v) => v.parse()?,
                    None => bail!("Usage: argc --argc-completions <SHELL> [CMDS]..."),
                };
                let script = crate::completions::generate(shell, &args[3..]);
                print!("{}", script);
            }
            "--argc-parallel" => {
                if args.len() <= 3 {
                    bail!("Usage: argc --argc-parallel <SCRIPT> <ARGS>...");
                }
                let shell = get_shell_path()?;
                let (source, cmd_args) = parse_script_args(&args[2..])?;
                if !source.contains("--argc-eval") {
                    bail!("Parallel only available for argc based scripts.")
                }
                let script_file = args[2].clone();
                parallel::parallel(&shell, &script_file, &cmd_args[1..])?;
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
                bail!("Unknown option `{argc_cmd}`")
            }
        }
        Ok(0)
    } else {
        let shell = get_shell_path()?;
        let (script_dir, script_file) = get_script_path(true)
            .ok_or_else(|| anyhow!("Argcfile not found, try `argc --argc-help` for help."))?;
        let mut envs = HashMap::new();
        if let Some(cwd) = get_current_dir() {
            envs.insert("ARGC_PWD".to_string(), escape_shell_words(&cwd));
        }
        let mut command = process::Command::new(shell);
        command
            .arg(&script_file)
            .args(&args[1..])
            .current_dir(script_dir)
            .envs(envs);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let err = command.exec();
            bail!("Failed to run `{err}`");
        }
        #[cfg(not(unix))]
        {
            let status = command
                .status()
                .with_context(|| format!("Failed to run `{}`", script_file.display()))?;
            Ok(status.code().unwrap_or_default())
        }
    }
}

fn run_compgen(mut args: Vec<String>) -> Option<()> {
    if args.len() < 6 {
        return None;
    };
    let shell: Shell = args.get(2).and_then(|v| v.parse().ok())?;
    if args[3].is_empty() {
        if args[4] == "argc" {
            if let Some((_, script_file)) = get_script_path(true) {
                args[3] = script_file.to_string_lossy().to_string();
            }
        } else if let Some(script_file) = search_completion_script(&mut args) {
            args[3] = script_file.to_string_lossy().to_string();
        } else if let Ok(script_file) = which(&args[4]) {
            args[3] = script_file.to_string_lossy().to_string();
        }
    }
    let no_color = std::env::var("NO_COLOR")
        .map(|v| v == "true" || v == "1")
        .unwrap_or_default();
    let output = if &args[4] == "argc" && (args[3].is_empty() || args[5].starts_with("--argc")) {
        let cmd_args = &args[4..];
        argc::compgen(shell, "", COMPLETION_SCRIPT, cmd_args, no_color).ok()?
    } else if args[3].is_empty() {
        let cmd_args = &args[4..];
        argc::compgen(shell, "", "# @arg path*", cmd_args, no_color).ok()?
    } else {
        let (source, cmd_args) = parse_script_args(&args[3..]).ok()?;
        argc::compgen(shell, &args[3], &source, &cmd_args[1..], no_color).ok()?
    };
    if !output.is_empty() {
        println!("{output}");
    }
    Some(())
}

fn export_argc_variables(code: &str) -> String {
    let mut value = code
        .split('\n')
        .filter(|line| line.starts_with(argc::VARIABLE_PREFIX))
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(";");
    if value.is_empty() {
        return value;
    } else {
        value.push(';');
    }
    format!(
        "export ARGC_VARS={}\n",
        general_purpose::STANDARD.encode(value)
    )
}

fn retrive_argc_variables() -> Option<String> {
    let value = env::var("ARGC_VARS").ok()?;
    let value = general_purpose::STANDARD.decode(value).ok()?;
    String::from_utf8(value).ok()
}

fn get_argc_help() -> String {
    let about = concat!(
        env!("CARGO_PKG_DESCRIPTION"),
        " - ",
        env!("CARGO_PKG_REPOSITORY")
    );
    format!(
        r###"{about}

USAGE:
    argc --argc-eval <SCRIPT> [ARGS]...             Use `eval "$(argc --argc-eval "$0" "$@")"`
    argc --argc-create [TASKS]...                   Create a boilerplate argcfile
    argc --argc-completions <SHELL> [CMDS]...       Generate shell completion scripts
    argc --argc-compgen <SHELL> <SCRIPT> <ARGS>...  Dynamically generating completion candidates
    argc --argc-export <SCRIPT>                     Export command line definitions as json
    argc --argc-parallel <SCRIPT> <ARGS>...         Execute argc functions in parallel
    argc --argc-script-path                         Print current argcfile path
    argc --argc-help                                Print help information
    argc --argc-version                             Print version information
"###
    )
}

fn get_argc_version() -> String {
    let name = env!("CARGO_CRATE_NAME");
    let version = env!("CARGO_PKG_VERSION");
    format!("{name} {version}")
}

fn export_dir_vars(path: &str) -> String {
    if let (Some(argc_script_dir), Some(cwd)) = (
        get_argc_script_dir(path),
        env::var("ARGC_PWD").ok().or_else(get_current_dir),
    ) {
        let cd = if argc_script_dir != cwd {
            format!("cd {}\n", escape_shell_words(&argc_script_dir))
        } else {
            String::new()
        };
        format!("{}export ARGC_PWD={}\n", cd, escape_shell_words(&cwd))
    } else {
        String::new()
    }
}

fn get_argc_script_dir(path: &str) -> Option<String> {
    if candidate_script_names().iter().all(|v| !path.ends_with(v)) {
        return None;
    }
    let script_file = fs::canonicalize(Path::new(path)).ok()?;
    let script_dir = script_file.parent()?;
    Some(script_dir.display().to_string())
}
