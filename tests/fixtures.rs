use std::collections::HashMap;
use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin;
use assert_fs::fixture::{ChildPath, TempDir};
use assert_fs::prelude::*;
use rstest::fixture;

#[allow(dead_code)]
pub type Error = Box<dyn std::error::Error>;

pub const SCRIPT_PATHS: [&str; 8] = [
    "dir1/Argcfile.sh",
    "dir1/subdir1/Argcfile.sh",
    "dir1/subdir1/subdirdir1/EMPTY",
    "dir2/argcfile.sh",
    "dir3/ARGCFILE.sh",
    "dir4/Argcfile",
    "dir5/argcfile",
    "dir6/ARGCFILE",
];

pub fn locate_script(script_path: &str) -> String {
    let mut spec_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    spec_path.push(script_path);
    spec_path.display().to_string()
}

#[fixture]
#[allow(dead_code)]
pub fn tmpdir() -> TempDir {
    assert_fs::TempDir::new().expect("Couldn't create a temp dir for tests")
}
/// Test fixture which creates a temporary directory with a few files and directories inside.
/// The directories also contain files.
#[fixture]
#[allow(dead_code)]
pub fn tmpdir_argcfiles() -> TempDir {
    let tmpdir = assert_fs::TempDir::new().expect("Couldn't create a temp dir for tests");
    for path in SCRIPT_PATHS {
        let cp = tmpdir_path(&tmpdir, path);
        if path == "EMPTY" {
            cp.write_str("").unwrap();
        } else {
            cp.write_str(&get_script(path)).unwrap();
        }
    }
    tmpdir
}

pub fn get_path_env_var() -> String {
    let argc_path = cargo_bin("argc");
    let argc_dir = argc_path.parent().unwrap();
    let path_env_var = std::env::var("PATH").unwrap();
    if cfg!(windows) {
        format!("{};{}", argc_dir.display(), path_env_var)
    } else {
        format!("{}:{}", argc_dir.display(), path_env_var)
    }
}

pub fn tmpdir_path(tmpdir: &TempDir, path: &str) -> ChildPath {
    let parts: Vec<&str> = path.split('/').collect();
    let cp = tmpdir.child(parts[0]);
    parts.iter().skip(1).fold(cp, |acc, part| acc.child(part))
}

pub fn create_argc_script(source: &str, name: &str) -> (String, String, assert_fs::NamedTempFile) {
    let script_content = patch_argc_bin(source);
    let script_file = assert_fs::NamedTempFile::new(name).unwrap();
    script_file.write_str(&script_content).unwrap();
    let script_file = script_file.into_persistent();
    let script_path = script_file.to_string_lossy().to_string();
    (script_path, script_content, script_file)
}

pub fn patch_argc_bin(source: &str) -> String {
    let argc_path = get_argc_path();
    if source.contains("--argc-eval") {
        source.replace("argc --argc-eval", &format!("{argc_path} --argc-eval"))
    } else {
        format!(
            r###"{source}
eval "$({argc_path} --argc-eval "$0" "$@")"
"###,
        )
    }
}

pub fn build_script(script_dir: &TempDir, source: &str) -> PathBuf {
    let has_fn = source.contains("()");
    let patched_source = source.replace(
        "{ :; }",
        r#"{
    ( set -o posix ; set ) | grep ^argc_
    echo "$argc__fn" "$@"
}"#,
    );
    let mut output = argc::build(&patched_source, "prog").unwrap();
    if !has_fn {
        output.push_str("\n( set -o posix ; set ) | grep ^argc_");
    }
    let script_file = script_dir.child("prog.sh");

    script_file.write_str(&output).unwrap();
    script_file.path().to_path_buf()
}

pub fn run_script<T: AsRef<Path>>(
    script_path: T,
    args: &[String],
    envs: &[(&str, &str)],
) -> String {
    let path_env_var = get_path_env_var();
    let envs: HashMap<&str, &str> = envs.iter().cloned().collect();
    let shell_path = argc::utils::get_shell_path().unwrap();
    let output = std::process::Command::new(shell_path)
        .arg(script_path.as_ref())
        .args(args)
        .env("PATH", path_env_var.clone())
        .envs(envs)
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    format!("{stdout}{stderr}")
}

fn get_argc_path() -> String {
    let argc_path = assert_cmd::cargo::cargo_bin("argc");
    let cwd = std::env::current_dir().unwrap();
    let base_path = argc_path.strip_prefix(cwd).unwrap();
    let base_path = base_path.display().to_string();
    base_path.replace('\\', "/")
}

fn get_script(name: &str) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

main() {{
  echo "{name}"
}}

# @cmd
task1() {{
    sleep $1
}}

# @cmd
cmd1() {{
    sleep 2
    echo cmd1 "$@" 
    (set -o posix; set)  | grep argc_
    env | grep ARGC_ | grep 'ARGC_PARALLEL\|ARGC_VARS' | sort
    echo cmd1 "$@"  >&2
}}

# @cmd
cmd2() {{
    sleep 2
    echo cmd2 "$@" 
    (set -o posix; set)  | grep argc_
    env | grep ARGC_ | grep 'ARGC_PARALLEL\|ARGC_VARS' | sort
    echo cmd2 "$@"  >&2
}}

# @cmd
# @option --oa
task2() {{
    argc --argc-parallel "$0" cmd1 abc ::: cmd2
}}

eval "$(argc --argc-eval "$0" "$@")"
"#
    )
}
