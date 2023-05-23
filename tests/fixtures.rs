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

pub fn locate_script(script_name: &str) -> String {
    let mut spec_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    spec_path.push("tests");
    spec_path.push("scripts");
    spec_path.push(script_name);
    spec_path.display().to_string()
}

/// Test fixture which creates a temporary directory with a few files and directories inside.
/// The directories also contain files.
#[fixture]
#[allow(dead_code)]
pub fn tmpdir() -> TempDir {
    let tmpdir = assert_fs::TempDir::new().expect("Couldn't create a temp dir for tests");
    for path in SCRIPT_PATHS {
        let cp = tmpdir_path(&tmpdir, path);
        if path.ends_with("EMPTY") {
            cp.write_str("").unwrap();
        } else {
            cp.write_str(&get_script(path)).unwrap();
        }
    }
    tmpdir
}

#[fixture]
#[allow(dead_code)]
pub fn tmpdir_bare() -> TempDir {
    assert_fs::TempDir::new().expect("Couldn't create a temp dir for tests")
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
    let argc_path = cargo_bin("argc");
    let argc_path = argc_path.to_str().unwrap();
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

eval "$(argc --argc-eval "$0" "$@")"
"#
    )
}
