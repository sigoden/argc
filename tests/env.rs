use rstest::rstest;

use crate::{fixtures::get_path_env_var, locate_script};
use std::process::Command;

macro_rules! snapshot_env {
    (
        args: [$($arg:literal),*],
        envs: {$($key:literal : $value:literal),*}

    ) => {
        let script_path = locate_script("examples/envs.sh");
        let path_env_var = get_path_env_var();
        let mut command = Command::new("bash");
        command.arg(script_path).env("PATH", path_env_var.clone());
        $(
            command.arg($arg);
        )*
        $(
            command.env($key, $value);
        )*

        let output = command.output().unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        insta::assert_snapshot!(format!(r#"
STDOUT
{stdout}

STDERR
{stderr}
"#));
    };
}

#[rstest]
fn env_help() {
    snapshot_env!(args: ["-h"], envs: {});
}

#[rstest]
fn env_help_subcmd() {
    snapshot_env!(args: ["run", "-h"], envs: {});
}

#[rstest]
fn env_missing() {
    snapshot_env!(args: [], envs: {});
}

#[rstest]
fn env_choice() {
    snapshot_env!(args: [], envs: {"TEST_EB": "1", "TEST_ECA": "val"});
}

#[rstest]
fn env_choice_fn() {
    snapshot_env!(args: [], envs: {"TEST_EB": "1", "TEST_EFA": "val"});
}

#[rstest]
fn env_run() {
    snapshot_env!(args: [], envs: {"TEST_EB": "1"});
}
