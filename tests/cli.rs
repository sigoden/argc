use assert_cmd::prelude::*;
use assert_fs::fixture::PathChild;
use std::{process::Command, time::Instant};

use crate::fixtures::{
    get_path_env_var, locate_script, tmpdir, tmpdir_argcfiles, tmpdir_path, SCRIPT_PATHS,
};

#[test]
fn version() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-version")
        .assert()
        .stdout(predicates::str::contains(format!(
            "argc {}",
            env!("CARGO_PKG_VERSION")
        )))
        .success();
}

#[test]
fn help() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-help")
        .assert()
        .stdout(predicates::str::contains(env!("CARGO_PKG_DESCRIPTION")))
        .success();
}

#[test]
fn create() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .assert()
        .success();
    assert!(tmpdir.path().join("Argcfile.sh").exists());
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .assert()
        .success();
}

#[test]
fn create_with_tasks() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .args(["foo", "bar"])
        .assert()
        .success();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .arg("bar")
        .assert()
        .stdout(predicates::str::contains("TODO bar"))
        .success();
}

#[test]
fn run() {
    let path_env_var = get_path_env_var();
    let path = locate_script("examples/demo.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg(path)
        .env("PATH", path_env_var)
        .assert()
        .stderr(predicates::str::contains("USAGE: demo"))
        .success();
}

#[test]
fn build_stdout() {
    let path = locate_script("examples/demo.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(path)
        .assert()
        .stdout(predicates::str::contains("# ARGC-BUILD"))
        .success();
}

#[test]
fn run_build() {
    let path = locate_script("examples/strict.sh");
    let tmpdir = tmpdir();
    let outpath = tmpdir.join("strict.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(&outpath)
        .assert()
        .success();

    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg(&outpath)
        .args([
            "--fa",
            "--oa",
            "oa1",
            "--of=of1,of2",
            "--oca=a",
            "--ofa",
            "abc",
        ])
        .assert()
        .stdout(predicates::str::contains("argc__fn=main"))
        .success();
}

#[test]
fn mangen() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outdir = tmpdir.to_path_buf();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg(&path)
        .arg(&outdir)
        .assert()
        .success();
    let manpath = outdir.join("demo.1");
    let script = std::fs::read_to_string(manpath).unwrap();
    assert!(script.contains(".TH DEMO 1"));
}

#[test]
fn completions() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-completions", "bash", "mycmd1", "mycmd2"])
        .assert()
        .stdout(predicates::str::contains(r#"argc mycmd1 mycmd2"#))
        .success();
}

#[test]
fn compgen_args() {
    let path = locate_script("examples/args.sh");
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-compgen")
        .arg("fish")
        .arg(path)
        .args(["args", "cmd_arg_with_choice_fn", ""])
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("abc\ndef\nghi"))
        .success();
}

#[test]
fn compgen_options() {
    let path = locate_script("examples/options.sh");
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-compgen")
        .arg("fish")
        .arg(path)
        .args(["args", "test1", "--cc", ""])
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("abc\ndef\nghi"))
        .success();
}

#[test]
fn compgen_argc() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "fish", "", "argc", "--argc-compgen", ""])
        .assert()
        .stdout(predicates::str::contains("zsh"))
        .success();
}

#[test]
fn compgen_kind() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args([
            "--argc-compgen",
            "fish",
            argc::COMPGEN_KIND_SYMBOL,
            "shell",
            "",
        ])
        .assert()
        .stdout(predicates::str::contains("zsh"))
        .success();
}

#[test]
fn export() {
    let path = locate_script("examples/options.sh");
    let output = Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-export")
        .arg(path)
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    insta::assert_snapshot!(stdout);
}

#[test]
fn parallel() {
    let tmpdir = tmpdir_argcfiles();
    let path_env_var = get_path_env_var();
    let args = ["task2", "--oa", "3"];
    let start_time = Instant::now();
    let output = Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir_path(&tmpdir, "dir1"))
        .env("PATH", path_env_var)
        .args(args)
        .output()
        .unwrap();

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = format!(
        r###"ARGS: {args:?}

STDOUT:
{stdout}

STDERR:
{stderr}

"###
    );
    assert!(elapsed_time.as_millis() < 3999);
    insta::assert_snapshot!(text);
}

#[test]
fn script_path() {
    let tmpdir = tmpdir_argcfiles();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-script-path")
        .current_dir(tmpdir.child("dir1").path())
        .assert()
        .stdout(predicates::str::contains(
            tmpdir_path(&tmpdir, "dir1/Argcfile.sh")
                .display()
                .to_string(),
        ))
        .success();
}

#[test]
fn shell_path() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-shell-path")
        .assert()
        .stdout(predicates::str::contains("bash"))
        .success();
}

#[test]
fn run_argcfile() {
    let tmpdir = tmpdir_argcfiles();
    let path_env_var = get_path_env_var();
    for path in SCRIPT_PATHS {
        if path.ends_with("EMPTY") {
            continue;
        }
        Command::new(assert_cmd::cargo::cargo_bin!())
            .current_dir(tmpdir_path(&tmpdir, path).path().parent().unwrap())
            .env("PATH", path_env_var.clone())
            .assert()
            .stdout(predicates::str::contains(path))
            .success();
    }

    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir_path(&tmpdir, "dir1/subdir1/subdirdir1"))
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("dir1/subdir1/Argcfile.sh"))
        .success();
}

#[test]
fn eval_missing_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .assert()
        .stdout(predicates::str::contains("Error:"))
        .success();
}

#[test]
fn eval_nonexistent_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .arg("/nonexistent/path/to/script.sh")
        .assert()
        .stdout(predicates::str::contains("Error:"))
        .stdout(predicates::str::contains("Failed to load script"))
        .success();
}

#[test]
fn eval_invalid_script_path() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .arg("")
        .assert()
        .stdout(predicates::str::contains("Error:"))
        .success();
}

#[test]
fn run_missing_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .assert()
        .stderr(predicates::str::contains("No script file provided"))
        .failure();
}

#[test]
fn run_nonexistent_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg("/nonexistent/path/to/script.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to run"))
        .failure();
}

#[test]
fn build_missing_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .assert()
        .stderr(predicates::str::contains("No script file provided"))
        .failure();
}

#[test]
fn build_nonexistent_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg("/nonexistent/path/to/script.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to load script"))
        .failure();
}

#[test]
fn build_to_directory() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outdir = tmpdir.to_path_buf();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(&outdir)
        .assert()
        .success();
    let outfile = outdir.join("demo.sh");
    assert!(outfile.exists());
    let content = std::fs::read_to_string(outfile).unwrap();
    assert!(content.contains("# ARGC-BUILD"));
}

#[test]
fn build_to_file() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outfile = tmpdir.join("custom.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(&outfile)
        .assert()
        .success();
    assert!(outfile.exists());
    let content = std::fs::read_to_string(outfile).unwrap();
    assert!(content.contains("# ARGC-BUILD"));
}

#[test]
fn mangen_missing_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .assert()
        .stderr(predicates::str::contains("No script file provided"))
        .failure();
}

#[test]
fn mangen_nonexistent_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg("/nonexistent/path/to/script.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to load script"))
        .failure();
}

#[test]
fn mangen_missing_outdir() {
    let path = locate_script("examples/demo.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg(&path)
        .assert()
        .stderr(predicates::str::contains("No output dir"))
        .failure();
}

#[test]
fn mangen_invalid_outdir_is_file() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let file_path = tmpdir.child("notadir");
    file_path.write_str("content").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg(&path)
        .arg(file_path.path())
        .assert()
        .stderr(predicates::str::contains("Not an directory"))
        .failure();
}

#[test]
fn completions_missing_shell() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-completions")
        .assert()
        .stderr(predicates::str::contains("Usage: argc --argc-completions"))
        .failure();
}

#[test]
fn completions_invalid_shell() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-completions")
        .arg("invalid_shell")
        .assert()
        .stderr(predicates::str::contains("invalid or missing"))
        .failure();
}

#[test]
fn completions_all_shells() {
    let shells = ["bash", "zsh", "fish", "powershell", "elvish", "nushell", "xonsh", "tcsh"];
    for shell in shells {
        Command::new(assert_cmd::cargo::cargo_bin!())
            .args(["--argc-completions", shell, "testcmd"])
            .assert()
            .stdout(predicates::str::contains("testcmd"))
            .success();
    }
}

#[test]
fn compgen_missing_args() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-compgen")
        .assert()
        .success();
}

#[test]
fn compgen_invalid_shell() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "invalid_shell", "", "cmd", ""])
        .assert()
        .success();
}

#[test]
fn compgen_nonexistent_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "bash", "/nonexistent/script.sh", "prog", ""])
        .assert()
        .success();
}

#[test]
fn export_missing_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-export")
        .assert()
        .stderr(predicates::str::contains("No script file provided"))
        .failure();
}

#[test]
fn export_nonexistent_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-export")
        .arg("/nonexistent/path/to/script.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to load script"))
        .failure();
}

#[test]
fn parallel_missing_args() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-parallel")
        .assert()
        .stderr(predicates::str::contains("Usage: argc --argc-parallel"))
        .failure();
}

#[test]
fn parallel_missing_script_args() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-parallel")
        .arg("script.sh")
        .assert()
        .stderr(predicates::str::contains("Usage: argc --argc-parallel"))
        .failure();
}

#[test]
fn parallel_nonexistent_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-parallel", "/nonexistent/script.sh", "cmd"])
        .assert()
        .stderr(predicates::str::contains("Failed to load script"))
        .failure();
}

#[test]
fn parallel_non_argc_script() {
    let tmpdir = tmpdir();
    let script_file = tmpdir.child("script.sh");
    script_file.write_str("#!/bin/bash\necho hello\n").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-parallel", script_file.path().to_str().unwrap(), "cmd"])
        .assert()
        .stderr(predicates::str::contains("Parallel only available for argc based scripts"))
        .failure();
}

#[test]
fn script_path_not_found() {
    let tmpdir = tmpdir();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-script-path")
        .current_dir(tmpdir.path())
        .assert()
        .stderr(predicates::str::contains("Argcfile not found"))
        .failure();
}

#[test]
fn unknown_argc_option() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-unknown")
        .assert()
        .stderr(predicates::str::contains("Unknown option"))
        .failure();
}

#[test]
fn argcfile_not_found() {
    let tmpdir = tmpdir();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .assert()
        .stderr(predicates::str::contains("Argcfile not found"))
        .failure();
}

#[test]
fn create_already_exists() {
    let tmpdir = tmpdir();
    let script_file = tmpdir.child("Argcfile.sh");
    script_file.write_str("#!/bin/bash\n").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .arg("--argc-create")
        .assert()
        .stderr(predicates::str::contains("Already exist"))
        .failure();
}

#[test]
fn create_with_custom_env_script_name() {
    let tmpdir = tmpdir();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("ARGC_SCRIPT_NAME", "MyTasks")
        .arg("--argc-create")
        .assert()
        .success();
    assert!(tmpdir.path().join("MyTasks").exists());
}

#[test]
fn version_format() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-version")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.starts_with("argc "));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_contains_all_options() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-help")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let options = [
        "--argc-eval",
        "--argc-run",
        "--argc-create",
        "--argc-build",
        "--argc-mangen",
        "--argc-completions",
        "--argc-compgen",
        "--argc-export",
        "--argc-parallel",
        "--argc-script-path",
        "--argc-shell-path",
        "--argc-help",
        "--argc-version",
    ];
    for option in options {
        assert!(stdout.contains(option), "Missing option {} in help", option);
    }
}

#[test]
fn run_with_subcommand() {
    let path = locate_script("examples/demo.sh");
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg(&path)
        .args(["upload", "test.txt"])
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("upload"))
        .stdout(predicates::str::contains("test.txt"))
        .success();
}

#[test]
fn run_with_help_flag() {
    let path = locate_script("examples/demo.sh");
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg(&path)
        .args(["--help"])
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("USAGE:"))
        .success();
}

#[test]
fn completions_with_multiple_commands() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-completions", "bash", "cmd1", "cmd2", "cmd3"])
        .assert()
        .stdout(predicates::str::contains("cmd1"))
        .stdout(predicates::str::contains("cmd2"))
        .stdout(predicates::str::contains("cmd3"))
        .success();
}

#[test]
fn compgen_argc_internal_commands() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "fish", "", "argc", "--argc-", ""])
        .assert()
        .stdout(predicates::str::contains("--argc-eval"))
        .success();
}

#[test]
fn compgen_kind_path() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args([
            "--argc-compgen",
            "fish",
            argc::COMPGEN_KIND_SYMBOL,
            "path",
            "",
        ])
        .assert()
        .success();
}

#[test]
fn export_with_subcommand() {
    let path = locate_script("examples/demo.sh");
    let output = Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-export")
        .arg(&path)
        .args(["upload"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("name"));
}

#[test]
fn mangen_creates_directory() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outdir = tmpdir.join("new_dir");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg(&path)
        .arg(&outdir)
        .assert()
        .success();
    assert!(outdir.exists());
    assert!(outdir.join("demo.1").exists());
}

#[test]
fn build_creates_output_directory() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outpath = tmpdir.join("new_dir").join("demo.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(&outpath)
        .assert()
        .success();
    assert!(outpath.exists());
}
