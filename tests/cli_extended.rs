use assert_cmd::prelude::*;
use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use std::process::Command;

use crate::fixtures::{get_path_env_var, locate_script, tmpdir};

// =============================================================================
// --argc-eval Tests
// =============================================================================

#[test]
fn eval_basic() {
    let path = locate_script("examples/demo.sh");
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .arg(path)
        .arg("test")
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("argc__fn=test"))
        .success();
}

#[test]
fn eval_missing_script() {
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("Error: No script file provided"))
        .success();
}

#[test]
fn eval_nonexistent_script() {
    let path_env_var = get_path_env_var();
    // For --argc-eval, errors are output via `echo "Error:..."` to stdout
    // and the command succeeds (exit 0) since it's generating shell code
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .arg("nonexistent_script_1234.sh")
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("Failed to load script"))
        .success();
}

#[test]
fn eval_with_help_flag() {
    let path = locate_script("examples/demo.sh");
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-eval")
        .arg(path)
        .arg("--help")
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("USAGE:"))
        .success();
}

// =============================================================================
// --argc-run Error Tests
// =============================================================================

#[test]
fn run_missing_script() {
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .env("PATH", path_env_var)
        .assert()
        .stderr(predicates::str::contains("No script file provided"))
        .failure();
}

#[test]
fn run_nonexistent_script() {
    let path_env_var = get_path_env_var();
    // For --argc-run:
    // Path absolutization may succeed even for non-existent files
    // So the actual error may come from either:
    // 1. Rust code (path issues) - outputs to stderr
    // 2. Shell execution (file not found) - outputs vary by platform
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg("nonexistent_script_1234.sh")
        .env("PATH", path_env_var)
        .assert()
        // The command should fail with non-zero exit code
        .code(predicates::ne(0));
}

#[test]
fn run_script_with_invalid_args() {
    let path = locate_script("examples/demo.sh");
    let path_env_var = get_path_env_var();
    // When invalid subcommand is passed, the script outputs error to stdout
    // via `echo` from --argc-eval error handling, and exits with code 1
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg(path)
        .arg("nonexistent_cmd")
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains(
            "error: Found argument 'nonexistent_cmd' which wasn't expected",
        ))
        .code(predicates::ne(0));
}

// =============================================================================
// --argc-build Error Tests
// =============================================================================

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
        .arg("nonexistent_script_1234.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to load script"))
        .failure();
}

#[test]
fn build_to_directory() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(tmpdir.path())
        .assert()
        .success();
    // When building to directory, the full filename including .sh suffix is used
    // from get_script_name(), not from cmd_args[0] which strips .sh
    assert!(tmpdir.path().join("demo.sh").exists());
}

#[test]
fn build_to_file_path() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let output_file = tmpdir.path().join("my_demo");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(&output_file)
        .assert()
        .success();
    // When building to a specific file path, that exact path is used
    assert!(output_file.exists());
    // Verify it doesn't add .sh suffix
    assert!(!output_file.with_extension("sh").exists());
}

#[test]
fn build_to_new_directory_with_trailing_slash() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    // Create a non-existent directory path with trailing slash
    let out_dir = tmpdir.path().join("new_dir/");
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(&path)
        .arg(out_dir)
        .assert()
        .success();
    // With trailing slash, it's treated as a directory and full filename is used
    assert!(tmpdir.path().join("new_dir/demo.sh").exists());
}

// =============================================================================
// --argc-mangen Error Tests
// =============================================================================

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
fn mangen_nonexistent_script() {
    let tmpdir = tmpdir();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg("nonexistent_script_1234.sh")
        .arg(tmpdir.path())
        .assert()
        .stderr(predicates::str::contains("Failed to load script"))
        .failure();
}

#[test]
fn mangen_file_as_outdir() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let file_path = tmpdir.path().join("not_a_dir");
    std::fs::write(&file_path, "test content").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-mangen")
        .arg(path)
        .arg(file_path)
        .assert()
        .stderr(predicates::str::contains("Not an directory"))
        .failure();
}

// =============================================================================
// --argc-completions Error Tests
// =============================================================================

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
        .args(["--argc-completions", "invalid_shell", "mycmd"])
        .assert()
        .stderr(predicates::str::contains(
            "The provided shell is either invalid or missing",
        ))
        .failure();
}

#[test]
fn completions_all_shells() {
    let shells = ["bash", "zsh", "fish", "powershell", "elvish", "nushell", "xonsh", "tcsh"];
    for shell in shells {
        Command::new(assert_cmd::cargo::cargo_bin!())
            .args(["--argc-completions", shell, "mycmd"])
            .assert()
            .success();
    }
}

// =============================================================================
// --argc-export Error Tests
// =============================================================================

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
        .arg("nonexistent_script_1234.sh")
        .assert()
        .stderr(predicates::str::contains("Failed to load script"))
        .failure();
}

// =============================================================================
// --argc-parallel Error Tests
// =============================================================================

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
fn parallel_missing_script_args() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-parallel", "script.sh"])
        .assert()
        .stderr(predicates::str::contains(
            "Usage: argc --argc-parallel <SCRIPT> <ARGS>...",
        ))
        .failure();
}

#[test]
fn parallel_nonexistent_script() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-parallel", "nonexistent_script_1234.sh", "cmd"])
        .assert()
        .stderr(predicates::str::contains("Failed to load script"))
        .failure();
}

#[test]
fn parallel_non_argc_script() {
    let tmpdir = tmpdir();
    let script_path = tmpdir.path().join("test.sh");
    std::fs::write(
        &script_path,
        r#"#!/usr/bin/env bash
echo "hello world"
"#,
    )
    .unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-parallel", script_path.to_str().unwrap(), "cmd"])
        .assert()
        .stderr(predicates::str::contains(
            "Parallel only available for argc based scripts.",
        ))
        .failure();
}

// =============================================================================
// --argc-script-path Error Tests
// =============================================================================

#[test]
fn script_path_missing() {
    let tmpdir = tmpdir();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-script-path")
        .current_dir(tmpdir.path())
        .assert()
        .stderr(predicates::str::contains("Argcfile not found"))
        .failure();
}

// =============================================================================
// Unknown Option Test
// =============================================================================

#[test]
fn unknown_argc_option() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-unknown-option")
        .assert()
        .stderr(predicates::str::contains("Unknown option `--argc-unknown-option`"))
        .failure();
}

// =============================================================================
// Default Behavior (no --argc-* prefix) - Missing Argcfile
// =============================================================================

#[test]
fn default_no_argcfile() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .assert()
        .stderr(predicates::str::contains(
            "Argcfile not found, try `argc --argc-help` for help.",
        ))
        .failure();
}

// =============================================================================
// Additional Edge Cases
// =============================================================================

#[test]
fn run_with_empty_script() {
    let tmpdir = tmpdir();
    let script_path = tmpdir.path().join("empty.sh");
    std::fs::write(&script_path, "").unwrap();
    let path_env_var = get_path_env_var();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-run")
        .arg(script_path)
        .env("PATH", path_env_var)
        .assert()
        .success();
}

#[test]
fn build_empty_script() {
    let tmpdir = tmpdir();
    let script_path = tmpdir.path().join("empty.sh");
    std::fs::write(&script_path, "").unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-build")
        .arg(script_path)
        .assert()
        .success();
}

#[test]
fn compgen_without_sufficient_args() {
    // This should not panic, should just output nothing
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "bash"])
        .assert()
        .success();
}

#[test]
fn compgen_with_empty_script_path() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .args(["--argc-compgen", "bash", "", "mycmd", ""])
        .assert()
        .success();
}

#[test]
fn help_output_structure() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-help")
        .assert()
        .stdout(predicates::str::contains("USAGE:"))
        .stdout(predicates::str::contains("--argc-"))
        .success();
}

#[test]
fn version_format() {
    Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-version")
        .assert()
        .stdout(predicates::str::is_match(r"^argc \d+\.\d+\.\d+").unwrap())
        .success();
}

#[test]
fn shell_path_exists() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!())
        .arg("--argc-shell-path")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let shell_path = stdout.trim();
    assert!(
        shell_path.contains("bash") || shell_path.contains("sh"),
        "Shell path should contain bash or sh: {}",
        shell_path
    );
}

#[test]
fn create_already_exists() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    
    // First create should succeed
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .assert()
        .success();
    
    // Second create should fail (file already exists)
    Command::new(assert_cmd::cargo::cargo_bin!())
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .arg("--argc-create")
        .assert()
        .stderr(predicates::str::contains("Already exist"))
        .failure();
}
