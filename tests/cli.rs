use assert_cmd::prelude::*;
use std::process::Command;

use crate::fixtures::{get_path_env_var, locate_script, tmpdir};

#[test]
fn version() {
    Command::cargo_bin("argc")
        .unwrap()
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
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-help")
        .assert()
        .stdout(predicates::str::contains(env!("CARGO_PKG_DESCRIPTION")))
        .success();
}

#[test]
fn build_stdout() {
    let path = locate_script("examples/demo.sh");
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-build")
        .arg(path)
        .assert()
        .stdout(predicates::str::contains("# ARGC-BUILD"))
        .success();
}

#[test]
fn build_outpath() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outpath = tmpdir.join("demo.sh");
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-build")
        .arg(&path)
        .arg(&outpath)
        .assert()
        .success();
    let script = std::fs::read_to_string(&outpath).unwrap();
    assert!(script.contains("# ARGC-BUILD"));
}

#[test]
fn completions() {
    Command::cargo_bin("argc")
        .unwrap()
        .args(["--argc-completions", "bash", "mycmd1", "mycmd2"])
        .assert()
        .stdout(predicates::str::contains(
            r#"complete -F _argc_completer -o nospace -o nosort \
    argc mycmd1 mycmd2"#,
        ))
        .success();
}

#[test]
fn compgen_args() {
    let path = locate_script("examples/args.sh");
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")
        .unwrap()
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
    Command::cargo_bin("argc")
        .unwrap()
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
    Command::cargo_bin("argc")
        .unwrap()
        .args(["--argc-compgen", "fish", "", "argc", "--argc-compgen", ""])
        .assert()
        .stdout(predicates::str::contains("zsh"))
        .success();
}

#[test]
fn export() {
    let path = locate_script("examples/options.sh");
    let output = Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-export")
        .arg(path)
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    insta::assert_snapshot!(stdout);
}
