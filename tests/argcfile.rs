use assert_fs::{fixture::PathChild, TempDir};
use rstest::rstest;

use crate::fixtures::{tmpdir, Error};
use assert_cmd::{cargo::cargo_bin, prelude::*};
use std::process::Command;

#[rstest]
fn argcfile(tmpdir: TempDir) -> Result<(), Error> {
    let argc_path = cargo_bin("argc");
    let argc_dir = argc_path.parent().unwrap();
    let mut path_env_var = std::env::var("PATH").unwrap();
    path_env_var = format!("{};{}", path_env_var, argc_dir.display());

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
        .env("PATH", path_env_var.clone())
        .assert()
        .stdout(predicates::str::contains("dir4-Argcfile.sh"))
        .success();

    Ok(())
}
