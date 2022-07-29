use anyhow::{anyhow, bail, Result};
use clap::{arg, ArgAction, Command};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process,
};
use which::which;

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
    let mut args: Vec<String> = vec![];
    let mut script_args: Vec<String> = vec![];
    for (i, arg) in std::env::args().enumerate() {
        if i == 0 {
            args.push(arg);
            continue;
        }
        if script_args.is_empty() && arg.starts_with("--argc-") {
            args.push(arg);
            continue;
        }
        script_args.push(arg);
    }
    let matches = Command::new(env!("CARGO_CRATE_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .about(concat!(
            env!("CARGO_PKG_DESCRIPTION"),
            " - ",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .arg(arg!(--"argc-eval" "Print code snippets for eval"))
        .arg(arg!(--"argc-completion" "Print bash completion script"))
        .arg(arg!(--"argc-version" "Print version information").action(ArgAction::Version))
        .arg(arg!(--"argc-help" "Print help information").action(ArgAction::Help))
        .arg(arg!([SCRIPT] "Specific script file"))
        .arg(arg!([ARGUMENTS]... "Arguments passed to script file"))
        .try_get_matches_from(&args)?;

    if matches.is_present("argc-completion") {
        let (source, cmd_args) = parse_script_args(&script_args)?;
        let cli = argc::Cli::new(&source);
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        cli.complete(cmd_args[0], &mut io::stdout())?;
    } else if matches.is_present("argc-eval") {
        let (source, cmd_args) = parse_script_args(&script_args)?;
        let cli = argc::Cli::new(&source);
        let cmd_args: Vec<&str> = cmd_args.iter().map(|v| v.as_str()).collect();
        match cli.run(&cmd_args)? {
            Ok(stdout) => {
                println!("{}", stdout)
            }
            Err(stderr) => {
                eprintln!("{}", stderr);
                println!("exit 1");
            }
        }
    } else {
        if env::var("ARGC_MODE").is_ok() {
            bail!("Recognized an infinite loop, did you forget to add the `--argc-eval` option in eval");
        }
        let shell = get_shell_path().ok_or_else(|| anyhow!("Not found shell"))?;
        let script_names = candicate_script_names();
        let (script_dir, script_file) = script_names
            .into_iter()
            .find_map(|v| get_script_path(&v))
            .ok_or_else(|| anyhow!("Not found script file"))?;
        let mut command = process::Command::new(&shell);
        command.arg(&script_file);
        command.args(&script_args);
        command.current_dir(script_dir);
        command.env("ARGC_MODE", "true");
        let status = command
            .status()
            .map_err(|err| anyhow!("Run `{}` throw {}", script_file.display(), err))?;
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
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or_else(|| anyhow!("Failed to get script name"))?;
    let mut cmd_args = vec![name.to_string()];
    cmd_args.extend(args);
    Ok((source, cmd_args))
}

fn get_script_path(name: &str) -> Option<(PathBuf, PathBuf)> {
    let mut dir = env::current_dir().ok()?;
    loop {
        let path = dir.join(name);
        if path.exists() {
            return Some((dir, path));
        }
        dir = dir.parent()?.to_path_buf();
    }
}

fn candicate_script_names() -> Vec<String> {
    let mut names = vec![];
    if let Ok(name) = env::var("ARGC_SCRIPT") {
        names.push(name.clone());
        if !name.ends_with(".sh") {
            names.push(format!("{}.sh", name));
        }
    }
    names.push("argcfile".into());
    names.push("Argcfile".into());
    names.push("argcfile.sh".into());
    names.push("Argcfile.sh".into());
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
    which("bash").ok()?
}
