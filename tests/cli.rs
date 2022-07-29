use assert_cmd::prelude::*;
use std::path::Path;
use std::process::Command;

#[test]
fn version() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-version")
        .assert()
        .stderr(predicates::str::contains("argc"))
        .failure();
}

#[test]
fn help() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-help")
        .assert()
        .stderr(predicates::str::contains("argc [OPTIONS] [ARGS]"))
        .failure();
}

#[test]
fn completion() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-completion")
        .arg(Path::new("tests").join("spec.sh"))
        .assert()
        .stdout(predicates::str::contains("_spec() {"))
        .success();
}
