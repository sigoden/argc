use assert_cmd::prelude::*;
use assert_fs::fixture::PathChild;
use std::process::Command;

use crate::fixtures::{get_path_env_var, locate_script, tmpdir, tmpdir_argcfiles, tmpdir_path, SCRIPT_PATHS};

#[test]
fn run_missing_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .assert()
        .stderr(predicates::str::contains("No script file provided"))
        .failure();
}

#[test]
fn run_invalid_script_path() {
    // --argc-run 对不存在的脚本路径会尝试执行，在 Unix 上会返回 "Failed to run"
    #[cfg(unix)]
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg("/nonexistent/path/script.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to run"))
        .failure();
    
    #[cfg(not(unix))]
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg("/nonexistent/path/script.sh")
        .assert()
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
fn build_invalid_script_path() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg("/nonexistent/path/script.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to load script at"))
        .failure();
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
fn mangen_missing_outdir() {
    let path = locate_script("examples/demo.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg(path)
        .assert()
        .stderr(predicates::str::contains("No output dir"))
        .failure();
}

#[test]
fn mangen_invalid_outdir() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let file_path = tmpdir.child("not_a_dir");
    file_path.write_str("").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg(path)
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
        .stderr(predicates::str::contains(
            "Usage: argc --argc-completions <SHELL> [CMDS]...",
        ))
        .failure();
}

#[test]
fn completions_invalid_shell() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-completions", "invalid_shell"])
        .assert()
        .stderr(predicates::str::contains(
            "The provided shell is either invalid or missing, must be one of"
        ))
        .failure();
}

#[test]
fn compgen_insufficient_args() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "fish"])
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
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
fn export_invalid_script_path() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-export")
        .arg("/nonexistent/path/script.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to load script at"))
        .failure();
}

#[test]
fn parallel_missing_args() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-parallel")
        .assert()
        .stderr(predicates::str::contains(
            "Usage: argc --argc-parallel <SCRIPT> <ARGS>...",
        ))
        .failure();
}

#[test]
fn parallel_non_argc_script() {
    let tmpdir = tmpdir();
    let script_file = tmpdir.child("plain_script.sh");
    script_file.write_str("#!/usr/bin/env bash\necho hello").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-parallel")
        .arg(script_file.path())
        .arg("arg1")
        .assert()
        .stderr(predicates::str::contains(
            "Parallel only available for argc based scripts.",
        ))
        .failure();
}

#[test]
fn script_path_not_found() {
    let tmpdir = tmpdir();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-script-path")
        .current_dir(tmpdir.path())
        .assert()
        .stderr(predicates::str::contains("Argcfile not found."))
        .failure();
}

#[test]
fn create_already_exists() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    let argcfile = tmpdir.child("Argcfile.sh");
    argcfile.write_str("#!/usr/bin/env bash").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .arg("--argc-create")
        .assert()
        .stderr(predicates::str::contains("Already exist"))
        .failure();
}

#[test]
fn create_with_tasks_success() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .args(["build", "test", "deploy"])
        .assert()
        .stdout(predicates::str::contains("Argcfile.sh has been successfully created"))
        .success();
    
    // Verify the created file contains all tasks
    let argcfile_content = std::fs::read_to_string(tmpdir.path().join("Argcfile.sh")).unwrap();
    assert!(argcfile_content.contains("build()"));
    assert!(argcfile_content.contains("test()"));
    assert!(argcfile_content.contains("deploy()"));
}

#[test]
fn unknown_argc_option() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-unknown")
        .assert()
        .stderr(predicates::str::contains("Unknown option `--argc-unknown`"))
        .failure();
}

#[test]
fn default_no_argcfile() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .assert()
        .stderr(predicates::str::contains(
            "Argcfile not found, try `argc --argc-help` for help",
        ))
        .failure();
}

#[test]
fn eval_error_output() {
    let tmpdir = tmpdir();
    let script_file = tmpdir.child("test.sh");
    script_file
        .write_str("#!/usr/bin/env bash\n# @option --foo\n# @option --foo")
        .unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .arg(script_file.path())
        .assert()
        .stdout(predicates::str::contains("Error:"))
        .stdout(predicates::str::contains("exit 1"))
        .success();
}

#[test]
fn build_with_dir_outpath() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outdir = tmpdir.child("output");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(outdir.path())
        .assert()
        .success();
    let built_script = outdir.child("demo.sh");
    assert!(built_script.exists());
}

#[test]
fn build_with_file_outpath() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outfile = tmpdir.child("myscript.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(outfile.path())
        .assert()
        .success();
    assert!(outfile.exists());
}

#[test]
fn run_argcfile_subdir() {
    let tmpdir = tmpdir_argcfiles();
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir_path(&tmpdir, "dir1/subdir1").path())
        .env("PATH", path_env_var)
        .arg("task1")
        .arg("0.1")
        .assert()
        .stdout(predicates::str::contains("dir1/subdir1/Argcfile.sh"))
        .success();
}

#[test]
fn shell_path_output() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-shell-path")
        .assert()
        .stdout(predicates::str::contains("bash"))
        .success();
}

#[test]
fn compgen_argc_empty_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "fish", "", "argc", "--argc-help", ""])
        .assert()
        .stdout(predicates::str::contains("--argc-help"))
        .success();
}

#[test]
fn compgen_kind_symbol() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args([
            "--argc-compgen",
            "fish",
            argc::COMPGEN_KIND_SYMBOL,
            "path",
            "/",
        ])
        .assert()
        .success();
}

#[test]
fn compgen_kind_shell() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args([
            "--argc-compgen",
            "fish",
            argc::COMPGEN_KIND_SYMBOL,
            "shell",
            "",
        ])
        .assert()
        .stdout(predicates::str::contains("bash"))
        .success();
}

#[test]
fn compgen_kind_command() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args([
            "--argc-compgen",
            "fish",
            argc::COMPGEN_KIND_SYMBOL,
            "command",
            "",
        ])
        .assert()
        .success();
}

#[test]
fn eval_success() {
    let tmpdir = tmpdir();
    let script_file = tmpdir.child("test.sh");
    script_file
        .write_str("#!/usr/bin/env bash\n# @flag --foo\n# @option --bar")
        .unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .arg(script_file.path())
        .args(["test", "--foo", "--bar", "value"])
        .assert()
        .stdout(predicates::str::contains("argc_foo=1"))
        .stdout(predicates::str::contains("argc_bar=value"))
        .success();
}

#[test]
fn eval_with_help() {
    let tmpdir = tmpdir();
    let script_file = tmpdir.child("test.sh");
    script_file
        .write_str("#!/usr/bin/env bash\n# @describe Test script\n# @flag --foo")
        .unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .arg(script_file.path())
        .args(["test", "--help"])
        .assert()
        .stdout(predicates::str::contains("USAGE:"))
        .success();
}

#[test]
fn run_with_args() {
    let path = locate_script("examples/args.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg(path)
        .args(["cmd_arg_with_choice_fn", "abc"])
        .assert()
        .success();
}

#[test]
fn run_help() {
    let path = locate_script("examples/demo.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg(path)
        .arg("--help")
        .assert()
        .stderr(predicates::str::contains("USAGE:"))
        .success();
}

#[test]
fn completions_multiple_commands() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-completions", "bash", "cmd1", "cmd2", "cmd3"])
        .assert()
        .stdout(predicates::str::contains("cmd1"))
        .stdout(predicates::str::contains("cmd2"))
        .stdout(predicates::str::contains("cmd3"))
        .success();
}

#[test]
fn completions_fish_shell() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-completions", "fish", "mycmd"])
        .assert()
        .stdout(predicates::str::contains("fish"))
        .success();
}

#[test]
fn completions_zsh_shell() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-completions", "zsh", "mycmd"])
        .assert()
        .stdout(predicates::str::contains("zsh"))
        .success();
}

#[test]
fn export_valid_script() {
    let path = locate_script("examples/demo.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-export")
        .arg(path)
        .assert()
        .stdout(predicates::str::contains("\"name\""))
        .stdout(predicates::str::contains("\"cmds\""))
        .success();
}

#[test]
fn mangen_creates_valid_manpage() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outdir = tmpdir.child("man");
    std::fs::create_dir_all(outdir.path()).unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg(path)
        .arg(outdir.path())
        .assert()
        .stdout(predicates::str::contains("saved"))
        .success();
    let manpage = outdir.child("demo.1");
    assert!(manpage.exists());
    let content = std::fs::read_to_string(manpage.path()).unwrap();
    assert!(content.contains(".TH DEMO 1"));
}

#[test]
fn build_creates_executable() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outfile = tmpdir.child("output.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(&outfile)
        .assert()
        .success();
    assert!(outfile.exists());
    let content = std::fs::read_to_string(outfile.path()).unwrap();
    assert!(content.contains("# ARGC-BUILD"));
}

#[test]
fn run_argcfile_with_args() {
    let tmpdir = tmpdir_argcfiles();
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir_path(&tmpdir, "dir1").path())
        .env("PATH", path_env_var)
        .args(["task1", "0.01"])
        .assert()
        .stdout(predicates::str::contains("dir1/Argcfile.sh"))
        .success();
}

#[test]
fn run_argcfile_recursive_search() {
    let tmpdir = tmpdir_argcfiles();
    let path_env_var = get_path_env_var();
    // From deep subdir, should find parent Argcfile
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir_path(&tmpdir, "dir1/subdir1/subdirdir1").path())
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("dir1/subdir1/Argcfile.sh"))
        .success();
}

#[test]
fn compgen_with_script_path() {
    let path = locate_script("examples/demo.sh");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "fish", path.as_str(), "demo", ""])
        .assert()
        .success();
}

#[test]
fn compgen_argc_subcommand() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "fish", "", "argc", "--argc-", ""])
        .assert()
        .stdout(predicates::str::contains("--argc-eval"))
        .stdout(predicates::str::contains("--argc-run"))
        .success();
}

#[test]
fn version_format() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-version")
        .assert()
        .stdout(predicates::str::contains("argc "))
        .stdout(predicates::str::contains("."))
        .success();
}

#[test]
fn help_contains_all_options() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-help")
        .assert()
        .stdout(predicates::str::contains("--argc-eval"))
        .stdout(predicates::str::contains("--argc-run"))
        .stdout(predicates::str::contains("--argc-build"))
        .stdout(predicates::str::contains("--argc-mangen"))
        .stdout(predicates::str::contains("--argc-completions"))
        .stdout(predicates::str::contains("--argc-compgen"))
        .stdout(predicates::str::contains("--argc-export"))
        .stdout(predicates::str::contains("--argc-parallel"))
        .stdout(predicates::str::contains("--argc-script-path"))
        .stdout(predicates::str::contains("--argc-shell-path"))
        .stdout(predicates::str::contains("--argc-help"))
        .stdout(predicates::str::contains("--argc-version"))
        .success();
}
