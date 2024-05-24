mod parallel;

use anyhow::{anyhow, bail, Context, Result};
use argc::{
    compgen_kind,
    utils::{escape_shell_words, is_true_value},
    CompKind, NativeRuntime, Runtime, Shell, COMPGEN_KIND_SYMBOL,
};
use base64::{engine::general_purpose, Engine as _};
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process,
};

const ARGC_SCRIPT_NAMES: [&str; 6] = [
    "Argcfile.sh",
    "Argcfile",
    "argcfile.sh",
    "argcfile",
    "ARGCFILE.sh",
    "ARGCFILE",
];

const ARGC_COMPLETION_SCRIPT: &str = include_str!("completion.sh");

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
    let runtime = NativeRuntime;
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
                let (source, _script_path, cmd_args) = parse_script_args(&args[2..])?;
                if cmd_args
                    .get(1)
                    .map(|v| v == parallel::PARALLEL_SYMBOL)
                    .unwrap_or_default()
                {
                    let cmd_args_len = cmd_args.len();
                    if cmd_args_len < 3 {
                        bail!("No parallel command")
                    }
                    let mut code = retrieve_argc_variables().unwrap_or_default();
                    let mut cmds = vec![cmd_args[2].to_string()];
                    cmds.extend(cmd_args[3..].iter().map(|v| escape_shell_words(v)));
                    code.push_str(&cmds.join(" "));
                    println!("{code}")
                } else {
                    let values = argc::eval(
                        runtime,
                        &source,
                        &cmd_args,
                        Some(&args[2]),
                        get_term_width(),
                    )?;
                    let dir_vars = export_dir_vars(&args[2]);
                    let code = argc::ArgcValue::to_bash(&values);
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
            "--argc-build" => {
                let (source, script_path, cmd_args) = parse_script_args(&args[2..])?;
                let script = argc::build(&source, &cmd_args[0], get_term_width())?;
                if let Some(outpath) = cmd_args.get(1) {
                    let script_name = get_script_name(&script_path)?;
                    let (outpath, new) = ensure_outpath(outpath, script_name)
                        .with_context(|| format!("Invalid output path '{outpath}'"))?;
                    fs::write(&outpath, script).with_context(|| {
                        format!("Failed to write script to '{}'", outpath.display())
                    })?;
                    if new {
                        set_permissions(&outpath).with_context(|| {
                            format!(
                                "Failed to set execute permission to '{}'",
                                outpath.display()
                            )
                        })?;
                    }
                } else {
                    print!("{}", script);
                }
            }
            "--argc-mangen" => {
                let (source, _script_path, cmd_args) = parse_script_args(&args[2..])?;
                let outdir = cmd_args.get(1).ok_or_else(|| anyhow!("No output dir"))?;
                let pages = argc::mangen(&source, &cmd_args[0])?;
                let outdir = ensure_outdir(outdir).with_context(|| "Invalid output dir")?;
                for (filename, page) in pages {
                    let outfile = outdir.join(filename);
                    fs::write(&outfile, page)
                        .with_context(|| format!("Failed to write '{}'", outfile.display()))?;
                    println!("saved {}", outfile.display());
                }
            }
            "--argc-completions" => {
                let shell: Shell = match args.get(2) {
                    Some(v) => v.parse()?,
                    None => bail!("Usage: argc --argc-completions <SHELL> [CMDS]..."),
                };
                let commands = [vec!["argc".to_string()], args[3..].to_vec()].concat();
                let script = argc::generate_completions(shell, &commands);
                print!("{}", script);
            }
            "--argc-compgen" => {
                run_compgen(runtime, args.to_vec());
            }
            "--argc-export" => {
                let (source, _script_path, cmd_args) = parse_script_args(&args[2..])?;
                let value = argc::export(&source, &cmd_args[0])?;
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
            "--argc-parallel" => {
                if args.len() <= 3 {
                    bail!("Usage: argc --argc-parallel <SCRIPT> <ARGS>...");
                }
                let shell = runtime.shell_path()?;
                let (source, script_path, cmd_args) = parse_script_args(&args[2..])?;
                if !source.contains("--argc-eval") {
                    bail!("Parallel only available for argc based scripts.")
                }
                parallel::parallel(runtime, &shell, &script_path, &cmd_args[1..])?;
            }
            "--argc-script-path" => {
                let (_, script_file) =
                    get_script_path(true).ok_or_else(|| anyhow!("Argcfile not found."))?;
                println!("{}", script_file.display());
            }
            "--argc-shell-path" => {
                let shell = runtime.shell_path()?;
                println!("{}", shell);
            }
            "--argc-help" => {
                println!("{}", get_argc_help(runtime)?)
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
        let shell = runtime.shell_path()?;
        let (script_dir, script_file) = get_script_path(true)
            .ok_or_else(|| anyhow!("Argcfile not found, try `argc --argc-help` for help."))?;
        let mut envs = HashMap::new();
        if let Some(cwd) = runtime.current_dir() {
            envs.insert("ARGC_PWD".to_string(), escape_shell_words(&cwd));
        }
        let script_file = script_file.display().to_string();
        let args = [vec![&script_file], args[1..].iter().collect()].concat();
        run_command(&shell, &args, envs, Some(&script_dir))
    }
}

fn run_command<T: AsRef<OsStr>>(
    prog: &str,
    args: &[T],
    envs: HashMap<String, String>,
    cwd: Option<&Path>,
) -> Result<i32> {
    let mut command = process::Command::new(prog);
    command.args(args).envs(envs);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
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

fn run_compgen(runtime: NativeRuntime, mut args: Vec<String>) -> Option<()> {
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
        } else if let Some(script_file) = runtime.which(&args[4]) {
            args[3] = script_file;
        }
    }
    let no_color = std::env::var("NO_COLOR")
        .map(|v| is_true_value(&v))
        .unwrap_or_default();
    let last_arg = args.last().map(|v| v.as_str()).unwrap_or_default();
    let output = if &args[4] == "argc" && (args[3].is_empty() || args[5].starts_with("--argc")) {
        if (args[5] == "--argc-completions" || args[5] == "--argc-compgen") && args.len() == 7 {
            compgen_kind(runtime, shell, CompKind::Shell, last_arg, no_color).ok()?
        } else if args[5] == "--argc-compgen" {
            compgen_kind(runtime, shell, CompKind::Path, last_arg, no_color).ok()?
        } else {
            argc::compgen(
                runtime,
                shell,
                "",
                ARGC_COMPLETION_SCRIPT,
                &args[4..],
                no_color,
            )
            .ok()?
        }
    } else if args[3] == COMPGEN_KIND_SYMBOL {
        let kind = CompKind::new(&args[4]);
        compgen_kind(runtime, shell, kind, last_arg, no_color).ok()?
    } else if args[3].is_empty() {
        // Fallback unknown command to path completion
        let last_arg = args.last().map(|v| v.as_str()).unwrap_or_default();
        compgen_kind(runtime, shell, CompKind::Path, last_arg, no_color).ok()?
    } else {
        let (source, script_path, cmd_args) = parse_script_args(&args[3..]).ok()?;
        argc::compgen(
            runtime,
            shell,
            &script_path,
            &source,
            &cmd_args[1..],
            no_color,
        )
        .ok()?
    };
    if !output.is_empty() {
        println!("{output}");
    }
    Some(())
}

fn export_argc_variables(code: &str) -> String {
    let mut value = code
        .split('\n')
        .filter(|line| line.starts_with(argc::utils::VARIABLE_PREFIX))
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

fn retrieve_argc_variables() -> Option<String> {
    let value = env::var("ARGC_VARS").ok()?;
    let value = general_purpose::STANDARD.decode(value).ok()?;
    String::from_utf8(value).ok()
}

fn get_argc_help(runtime: NativeRuntime) -> Result<String> {
    let about = concat!(
        env!("CARGO_PKG_DESCRIPTION"),
        " - ",
        env!("CARGO_PKG_REPOSITORY")
    );
    let values = argc::eval(
        runtime,
        ARGC_COMPLETION_SCRIPT,
        &["argc".into(), "--help".into()],
        None,
        None,
    )?;
    let argc_options = argc::ArgcValue::to_bash(&values);
    let argc_options = argc_options
        .lines()
        .map(|v| v.trim())
        .filter(|v| v.starts_with("--argc-"))
        .map(|v| format!("    argc {v}\n"))
        .collect::<Vec<String>>()
        .join("");
    let output = format!(
        r###"{about}

USAGE:
{argc_options}"###
    );
    Ok(output)
}

fn get_argc_version() -> String {
    let name = env!("CARGO_CRATE_NAME");
    let version = env!("CARGO_PKG_VERSION");
    format!("{name} {version}")
}

fn get_term_width() -> Option<usize> {
    env::var("TERM_WIDTH").ok().and_then(|v| v.parse().ok())
}

fn export_dir_vars(path: &str) -> String {
    if let (Some(argc_script_dir), Some(cwd)) = (
        get_argc_script_dir(path),
        env::var("ARGC_PWD")
            .ok()
            .or_else(|| env::current_dir().ok().map(|v| v.to_string_lossy().into())),
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
    let path = Path::new(path);
    let script_file = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let current_dir = env::current_dir().ok()?;
        current_dir.join(path)
    };
    let script_dir = script_file.parent()?;
    Some(script_dir.display().to_string())
}

fn parse_script_args(args: &[String]) -> Result<(String, String, Vec<String>)> {
    if args.is_empty() {
        bail!("No script provided");
    }
    let script_file = args[0].as_str();
    let script_file = normalize_script_path(script_file);
    let args: Vec<String> = args[1..].to_vec();
    let source = fs::read_to_string(&script_file)
        .with_context(|| format!("Failed to load script at '{}'", script_file))?;
    let name = get_script_name(&script_file)?;
    let name = name.strip_suffix(".sh").unwrap_or(name);
    let mut cmd_args = vec![name.to_string()];
    cmd_args.extend(args);
    Ok((source, script_file, cmd_args))
}

fn get_script_name(script_path: &str) -> Result<&str> {
    Path::new(script_path)
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| anyhow!("Failed to extract filename form '{}'", script_path))
}

fn generate_boilerplate(args: &[String]) -> String {
    let tasks = args
        .iter()
        .map(|cmd| {
            format!(
                r#"
# @cmd
{cmd}() {{
    echo TODO {cmd}
}}
"#
            )
        })
        .collect::<Vec<String>>()
        .join("");

    format!(
        r#"#!/usr/bin/env bash

set -e
{tasks}
# See more details at https://github.com/sigoden/argc
eval "$(argc --argc-eval "$0" "$@")"
"#
    )
}

fn get_script_path(recursive: bool) -> Option<(PathBuf, PathBuf)> {
    let candidates = candidate_script_names();
    let mut dir = env::current_dir().ok()?;
    loop {
        for name in candidates.iter() {
            let path = dir.join(name);
            if path.exists() {
                return Some((dir, path));
            }
        }
        if !recursive {
            return None;
        }
        dir = dir.parent()?.to_path_buf();
    }
}

fn candidate_script_names() -> Vec<String> {
    let mut names = vec![];
    if let Ok(name) = env::var("ARGC_SCRIPT_NAME") {
        names.push(name.clone());
        if !name.ends_with(".sh") {
            names.push(format!("{name}.sh"));
        }
    }
    names.extend(ARGC_SCRIPT_NAMES.into_iter().map(|v| v.to_string()));
    names
}

fn search_completion_script(args: &mut Vec<String>) -> Option<PathBuf> {
    let search_paths = env::var("ARGC_COMPLETIONS_PATH").ok()?;
    let cmd = Path::new(&args[4])
        .file_stem()
        .and_then(|v| v.to_str())?
        .to_string();
    let subcmd = if args.len() >= 7
        && args[5]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
    {
        Some(args[5].clone())
    } else {
        None
    };
    let mut handlers = vec![];
    #[cfg(not(target_os = "windows"))]
    let sep = ":";
    #[cfg(target_os = "windows")]
    let sep = ";";
    for path in search_paths
        .split(sep)
        .filter(|v| !v.is_empty() && *v != sep)
    {
        let path = path.to_string();
        let cmd = cmd.to_string();
        let subcmd = subcmd.clone();
        let handler = std::thread::spawn(move || {
            if let Some(subcmd) = subcmd {
                let mut search_path = PathBuf::from(path.clone());
                search_path.push(cmd.clone());
                search_path.push(format!("{subcmd}.sh"));
                if search_path.exists() {
                    return Some((search_path, true));
                }
            }
            let mut search_path = PathBuf::from(path);
            search_path.push(format!("{cmd}.sh"));
            if search_path.exists() {
                return Some((search_path, false));
            }
            None
        });
        handlers.push(handler);
    }
    let mut subcmd_script_path = None;
    let mut cmd_script_path = None;
    for handler in handlers {
        if let Ok(Some((path, is_subcmd))) = handler.join() {
            if is_subcmd {
                subcmd_script_path = Some(path);
                break;
            } else if cmd_script_path.is_none() {
                cmd_script_path = Some(path);
            }
        }
    }
    if let (Some(path), Some(subcmd)) = (subcmd_script_path, subcmd) {
        args.remove(4);
        args[4] = format!("{cmd}-{subcmd}");
        return Some(path);
    }
    if let Some(path) = cmd_script_path {
        return Some(path);
    }
    None
}

fn ensure_outpath(outpath: &str, script_name: &str) -> Result<(PathBuf, bool)> {
    match fs::metadata(outpath) {
        Ok(metadata) => {
            let outpath = Path::new(outpath);
            if metadata.is_dir() {
                let outpath = outpath.join(script_name);
                Ok((outpath, true))
            } else {
                Ok((outpath.to_path_buf(), false))
            }
        }
        Err(_) => {
            let is_dir = outpath.ends_with('/') || outpath.ends_with('\\');
            let mut outpath = PathBuf::from(outpath);
            if is_dir {
                fs::create_dir_all(&outpath)?;
                outpath.push(script_name);
            } else if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            };
            Ok((outpath, true))
        }
    }
}

fn ensure_outdir(outdir: &str) -> Result<PathBuf> {
    match fs::metadata(outdir) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Ok(PathBuf::from(outdir))
            } else {
                bail!("Not an directory")
            }
        }
        Err(_) => {
            fs::create_dir_all(outdir)?;
            Ok(PathBuf::from(outdir))
        }
    }
}

#[cfg(unix)]
fn set_permissions<T: AsRef<Path>>(path: T) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let path = path.as_ref();
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_permissions<T: AsRef<Path>>(_path: T) -> Result<()> {
    Ok(())
}

fn normalize_script_path(path: &str) -> String {
    if cfg!(windows)
        && env::var("MSYSTEM").is_ok()
        && path.len() >= 3
        && path.starts_with('/')
        && path.chars().nth(1).map(|v| v.is_ascii_alphabetic()) == Some(true)
        && path.chars().nth(2) == Some('/')
    {
        let drive = path.chars().nth(1).unwrap().to_uppercase();
        return format!("{}:{}", drive, &path[2..].replace('/', "\\"));
    }
    path.to_string()
}
