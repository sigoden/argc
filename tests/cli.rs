use assert_cmd::prelude::*;
use std::process::Command;

use crate::fixtures::{get_path_env_var, get_spec};

#[test]
fn version() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-version")
        .assert()
        .stderr(predicates::str::contains(format!(
            "argc {}",
            env!("CARGO_PKG_VERSION")
        )))
        .failure();
}

#[test]
fn help() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-help")
        .assert()
        .stderr(predicates::str::contains(env!("CARGO_PKG_DESCRIPTION")))
        .failure();
}

#[test]
fn compgen() {
    let (path, _) = get_spec();
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-compgen")
        .arg("bash")
        .arg(path)
        .arg("cmd_option_names --op11 ")
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("a1\na2\na3"))
        .success();
}
