use assert_fs::TempDir;
use rstest::rstest;

use crate::fixtures::{get_path_env_var, tmpdir2, Error};
use assert_cmd::prelude::*;
use std::process::Command;

#[rstest]
fn create(tmpdir2: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir2.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .assert()
        .success();
    assert!(tmpdir2.path().join("argcfile.sh").exists());
    Command::cargo_bin("argc")?
        .current_dir(tmpdir2.path())
        .env("PATH", path_env_var)
        .assert()
        .success();
    Ok(())
}

#[rstest]
fn create_with_tasks(tmpdir2: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir2.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .args(&["foo", "bar"])
        .assert()
        .success();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir2.path())
        .env("PATH", path_env_var)
        .arg("bar")
        .assert()
        .stdout(predicates::str::contains("Run bar"))
        .success();
    Ok(())
}
