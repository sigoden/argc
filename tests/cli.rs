use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn version() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--version")
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
        .arg("--help")
        .assert()
        .stderr(predicates::str::contains(env!("CARGO_PKG_DESCRIPTION")))
        .failure();
}
