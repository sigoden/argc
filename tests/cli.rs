use assert_cmd::prelude::*;
use std::path::Path;
use std::process::Command;

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
fn completion() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-completion")
        .arg(Path::new("tests").join("spec.sh"))
        .assert()
        .stdout(predicates::str::contains("_spec.sh() {"))
        .success();
}
