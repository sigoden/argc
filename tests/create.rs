use assert_fs::TempDir;
use rstest::rstest;

use crate::fixtures::{get_path_env_var, tmpdir_bare, Error};
use assert_cmd::prelude::*;
use std::process::Command;

#[rstest]
fn create(tmpdir_bare: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir_bare.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .assert()
        .success();
    assert!(tmpdir_bare.path().join("Argcfile.sh").exists());
    Command::cargo_bin("argc")?
        .current_dir(tmpdir_bare.path())
        .env("PATH", path_env_var)
        .assert()
        .success();
    Ok(())
}

#[rstest]
fn create_with_tasks(tmpdir_bare: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir_bare.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .args(["foo", "bar"])
        .assert()
        .success();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir_bare.path())
        .env("PATH", path_env_var)
        .arg("bar")
        .assert()
        .stdout(predicates::str::contains("To implement command: bar"))
        .success();
    Ok(())
}
