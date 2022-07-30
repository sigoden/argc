use assert_fs::{fixture::PathChild, TempDir};
use rstest::rstest;

use crate::fixtures::{get_path_env_var, tmpdir, Error};
use assert_cmd::prelude::*;
use std::process::Command;

#[rstest]
fn argcfile(tmpdir: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")?
        .current_dir(tmpdir.child("dir1").path())
        .env("PATH", path_env_var.clone())
        .assert()
        .stdout(predicates::str::contains("dir1-argcfile"))
        .success();

    Command::cargo_bin("argc")?
        .current_dir(tmpdir.child("dir1").child("subdir1").path())
        .env("PATH", path_env_var.clone())
        .assert()
        .stdout(predicates::str::contains("dir1-subdir1-Argcfile"))
        .success();

    Command::cargo_bin("argc")?
        .current_dir(
            tmpdir
                .child("dir1")
                .child("subdir1")
                .child("subsubdir1")
                .path(),
        )
        .env("PATH", path_env_var.clone())
        .assert()
        .stdout(predicates::str::contains("dir1-subdir1-Argcfile"))
        .success();

    Command::cargo_bin("argc")?
        .current_dir(tmpdir.child("dir2").path())
        .env("PATH", path_env_var.clone())
        .assert()
        .stdout(predicates::str::contains("dir2-argcfile.sh"))
        .success();

    Command::cargo_bin("argc")?
        .current_dir(tmpdir.child("dir3").path())
        .env("PATH", path_env_var.clone())
        .assert()
        .stdout(predicates::str::contains("dir3-Argcfile"))
        .success();

    Command::cargo_bin("argc")?
        .current_dir(tmpdir.child("dir4").path())
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("dir4-Argcfile.sh"))
        .success();

    Ok(())
}

#[rstest]
fn argcfile_path(tmpdir: TempDir) -> Result<(), Error> {
    Command::cargo_bin("argc")?
        .arg("--argc-argcfile")
        .current_dir(
            tmpdir
                .child("dir1")
                .child("subdir1")
                .child("subsubdir1")
                .path(),
        )
        .assert()
        .stdout(predicates::str::is_match("Argcfile$").unwrap())
        .success();
    Ok(())
}
