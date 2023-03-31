use assert_fs::{fixture::PathChild, TempDir};
use rstest::rstest;

use crate::fixtures::{get_path_env_var, tmpdir, Error};
use assert_cmd::prelude::*;
use std::{process::Command, thread, time::Duration};

fn kill(process_id: u32) {
    unsafe {
        libc::kill(process_id as i32, libc::SIGINT);
    }
}

#[rstest]
fn interrupt(tmpdir: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();

    let mut child = Command::cargo_bin("argc")?
        .current_dir(tmpdir.child("dir1").path())
        .env("PATH", path_env_var)
        .args(["task1", "2"])
        .spawn()
        .expect("argc invocation failed");

    thread::sleep(Duration::new(1, 0));

    kill(child.id());

    let status = child.wait().unwrap();

    assert_eq!(status.code(), Some(130));

    Ok(())
}
