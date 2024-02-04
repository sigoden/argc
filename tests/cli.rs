use assert_cmd::prelude::*;
use assert_fs::fixture::PathChild;
use std::{process::Command, time::Instant};

use crate::fixtures::{
    get_path_env_var, locate_script, tmpdir, tmpdir_argcfiles, tmpdir_path, SCRIPT_PATHS,
};

#[test]
fn version() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-version")
        .assert()
        .stdout(predicates::str::contains(format!(
            "argc {}",
            env!("CARGO_PKG_VERSION")
        )))
        .success();
}

#[test]
fn help() {
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-help")
        .assert()
        .stdout(predicates::str::contains(env!("CARGO_PKG_DESCRIPTION")))
        .success();
}

#[test]
fn create() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")
        .unwrap()
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .assert()
        .success();
    assert!(tmpdir.path().join("Argcfile.sh").exists());
    Command::cargo_bin("argc")
        .unwrap()
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .assert()
        .success();
}

#[test]
fn create_with_tasks() {
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")
        .unwrap()
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var.clone())
        .arg("--argc-create")
        .args(["foo", "bar"])
        .assert()
        .success();
    Command::cargo_bin("argc")
        .unwrap()
        .current_dir(tmpdir.path())
        .env("PATH", path_env_var)
        .arg("bar")
        .assert()
        .stdout(predicates::str::contains("To implement command: bar"))
        .success();
}

#[test]
fn build_stdout() {
    let path = locate_script("examples/demo.sh");
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-build")
        .arg(path)
        .assert()
        .stdout(predicates::str::contains("# ARGC-BUILD"))
        .success();
}

#[test]
fn build_outpath() {
    let path = locate_script("examples/demo.sh");
    let tmpdir = tmpdir();
    let outpath = tmpdir.join("demo.sh");
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-build")
        .arg(&path)
        .arg(&outpath)
        .assert()
        .success();
    let script = std::fs::read_to_string(&outpath).unwrap();
    assert!(script.contains("# ARGC-BUILD"));
}

#[test]
fn completions() {
    Command::cargo_bin("argc")
        .unwrap()
        .args(["--argc-completions", "bash", "mycmd1", "mycmd2"])
        .assert()
        .stdout(predicates::str::contains(
            r#"complete -F _argc_completer -o nospace -o nosort \
    argc mycmd1 mycmd2"#,
        ))
        .success();
}

#[test]
fn compgen_args() {
    let path = locate_script("examples/args.sh");
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-compgen")
        .arg("fish")
        .arg(path)
        .args(["args", "cmd_arg_with_choice_fn", ""])
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("abc\ndef\nghi"))
        .success();
}

#[test]
fn compgen_options() {
    let path = locate_script("examples/options.sh");
    let path_env_var = get_path_env_var();
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-compgen")
        .arg("fish")
        .arg(path)
        .args(["args", "test1", "--cc", ""])
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("abc\ndef\nghi"))
        .success();
}

#[test]
fn compgen_argc() {
    Command::cargo_bin("argc")
        .unwrap()
        .args(["--argc-compgen", "fish", "", "argc", "--argc-compgen", ""])
        .assert()
        .stdout(predicates::str::contains("zsh"))
        .success();
}

#[test]
fn export() {
    let path = locate_script("examples/options.sh");
    let output = Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-export")
        .arg(path)
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    insta::assert_snapshot!(stdout);
}

#[test]
fn parallel() {
    let tmpdir = tmpdir_argcfiles();
    let path_env_var = get_path_env_var();
    let args = ["task2", "--oa", "3"];
    let start_time = Instant::now();
    let output = Command::cargo_bin("argc")
        .unwrap()
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
    assert!(elapsed_time.as_millis() < 3999);
    insta::assert_snapshot!(text);
}

#[test]
fn script_path() {
    let tmpdir = tmpdir_argcfiles();
    Command::cargo_bin("argc")
        .unwrap()
        .arg("--argc-script-path")
        .current_dir(tmpdir.child("dir1").path())
        .assert()
        .stdout(predicates::str::contains(
            tmpdir_path(&tmpdir, "dir1/Argcfile.sh")
                .display()
                .to_string(),
        ))
        .success();
}

#[test]
fn run_argcfile() {
    let tmpdir = tmpdir_argcfiles();
    let path_env_var = get_path_env_var();
    for path in SCRIPT_PATHS {
        if path.ends_with("EMPTY") {
            continue;
        }
        Command::cargo_bin("argc")
            .unwrap()
            .current_dir(tmpdir_path(&tmpdir, path).path().parent().unwrap())
            .env("PATH", path_env_var.clone())
            .assert()
            .stdout(predicates::str::contains(path))
            .success();
    }

    Command::cargo_bin("argc")
        .unwrap()
        .current_dir(tmpdir_path(&tmpdir, "dir1/subdir1/subdirdir1"))
        .env("PATH", path_env_var)
        .assert()
        .stdout(predicates::str::contains("dir1/subdir1/Argcfile.sh"))
        .success();
}
