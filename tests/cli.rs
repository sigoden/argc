use assert_cmd::prelude::*;
use std::process::Command;

use crate::fixtures::{get_path_env_var, locate_script};

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
fn compgen() {
    let path = locate_script("args.sh");
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-compgen")
        .arg("bash")
        .arg(path)
        .arg("cmdj ")
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("abc\ndef\nxyz"))
        .success();
}
