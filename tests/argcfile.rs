use assert_fs::{fixture::PathChild, TempDir};
use rstest::rstest;

use crate::fixtures::{get_path_env_var, tmpdir_argcfiles, tmpdir_path, Error, SCRIPT_PATHS};
use assert_cmd::prelude::*;
use std::process::Command;

#[rstest]
fn argcfile(tmpdir_argcfiles: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();

    for path in SCRIPT_PATHS {
        if path.ends_with("EMPTY") {
            continue;
        }
        Command::cargo_bin("argc")?
            .current_dir(
                tmpdir_path(&tmpdir_argcfiles, path)
                    .path()
                    .parent()
                    .unwrap(),
            )
            .env("PATH", path_env_var.clone())
            .assert()
            .stdout(predicates::str::contains(path))
            .success();
    }

    Command::cargo_bin("argc")?
        .current_dir(tmpdir_path(&tmpdir_argcfiles, "dir1/subdir1/subdirdir1"))
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("dir1/subdir1/Argcfile.sh"))
        .success();

    Ok(())
}

#[rstest]
fn argcfile_path(tmpdir_argcfiles: TempDir) -> Result<(), Error> {
    Command::cargo_bin("argc")?
        .arg("--argc-script-path")
        .current_dir(tmpdir_argcfiles.child("dir1").path())
        .assert()
        .stdout(predicates::str::contains(
            tmpdir_path(&tmpdir_argcfiles, "dir1/Argcfile.sh")
                .display()
                .to_string(),
        ))
        .success();
    Ok(())
}
