use assert_cmd::cargo::cargo_bin;
use assert_fs::fixture::TempDir;
use assert_fs::prelude::*;
use rstest::fixture;

#[allow(dead_code)]
pub type Error = Box<dyn std::error::Error>;

/// Test fixture which creates a temporary directory with a few files and directories inside.
/// The directories also contain files.
#[fixture]
#[allow(dead_code)]
pub fn tmpdir() -> TempDir {
    let tmpdir = assert_fs::TempDir::new().expect("Couldn't create a temp dir for tests");
    tmpdir
        .child("dir1")
        .child("argcfile")
        .write_str(&get_argc_file("dir1-argcfile"))
        .unwrap();
    tmpdir
        .child("dir1")
        .child("subdir1")
        .child("Argcfile")
        .write_str(&get_argc_file("dir1-subdir1-Argcfile"))
        .unwrap();
    tmpdir
        .child("dir1")
        .child("subdir1")
        .child("subsubdir1")
        .child("EMPTY")
        .write_str("")
        .unwrap();
    tmpdir
        .child("dir1")
        .child("subdir1")
        .child("subsubdir1")
        .child("EMPTY")
        .write_str("")
        .unwrap();
    tmpdir
        .child("dir2")
        .child("argcfile.sh")
        .write_str(&get_argc_file("dir2-argcfile.sh"))
        .unwrap();
    tmpdir
        .child("dir3")
        .child("Argcfile")
        .write_str(&get_argc_file("dir3-Argcfile"))
        .unwrap();
    tmpdir
        .child("dir4")
        .child("Argcfile.sh")
        .write_str(&get_argc_file("dir4-Argcfile.sh"))
        .unwrap();
    tmpdir
}

pub fn get_path_env_var() -> String {
    let argc_path = cargo_bin("argc");
    let argc_dir = argc_path.parent().unwrap();
    let path_env_var = std::env::var("PATH").unwrap();
    if cfg!(windows) {
        format!("{};{}", path_env_var, argc_dir.display())
    } else {
        format!("{}:{}", path_env_var, argc_dir.display())
    }
}

fn get_argc_file(name: &str) -> String {
    format!(
        r#"
set -euo pipefail

main() {{
  echo "{name}"
}}

echo $PATH
eval "$(argc --argc-eval $0 "$@")"
"#
    )
}
