use assert_fs::TempDir;
use rstest::rstest;

use crate::fixtures::{get_path_env_var, tmpdir, Error};
use assert_cmd::prelude::*;
use std::process::Command;

#[rstest]
fn create(tmpdir: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .assert()
        .success();
    assert!(tmpdir.path().join("Argcfile.sh").exists());
    Command::cargo_bin("argc")?
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .assert()
        .success();
    Ok(())
}

#[rstest]
fn create_with_tasks(tmpdir: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .args(["foo", "bar"])
        .assert()
        .success();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .arg("bar")
        .assert()
        .stdout(predicates::str::contains("To implement command: bar"))
        .success();
    Ok(())
}
