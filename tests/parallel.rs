use assert_fs::TempDir;
use rstest::rstest;

use crate::fixtures::{get_path_env_var, tmpdir, tmpdir_path, Error};
use assert_cmd::prelude::*;
use std::{process::Command, time::Instant};

#[rstest]
fn run(tmpdir: TempDir) -> Result<(), Error> {
    let path_env_var = get_path_env_var();
    let args = ["task2", "--oa", "3"];
    let start_time = Instant::now();
    let output = Command::cargo_bin("argc")?
        .current_dir(tmpdir_path(&tmpdir, "dir1"))
        .env("PATH", path_env_var)
        .args(args)
        .output()
        .unwrap();

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = format!(
        r###"ARGS: {args:?}

STDOUT:
{stdout}

STDERR:
{stderr}

"###
    );
    assert!(elapsed_time.as_millis() < 3000);
    insta::assert_snapshot!(text);

    Ok(())
}
