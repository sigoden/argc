use crate::fixtures::argc_bin;

use predicates::prelude::*;

fn script_path() -> String {
    crate::fixtures::locate_script("examples/external.sh")
}

#[test]
fn help_shows_external_commands() {
    argc_bin()
        .arg("--argc-eval")
        .arg(script_path())
        .arg("--help")
        .assert()
        .stdout(predicate::str::contains("EXTERNAL COMMANDS:"))
        .stdout(predicate::str::contains("bar"))
        .stdout(predicate::str::contains("foo"))
        .stdout(predicate::str::contains(
            r#"An external subcommand "bar" with options"#,
        ))
        .stdout(predicate::str::contains(r#"An external subcommand "foo""#))
        .success();
}

#[test]
fn internal_subcommand_still_works() {
    argc_bin()
        .arg("--argc-eval")
        .arg(script_path())
        .arg("builtin")
        .arg("hello")
        .assert()
        .stdout(predicate::str::contains("argc__fn=builtin"))
        .success();
}

#[test]
fn external_external_subcommand_generates_bash_call() {
    argc_bin()
        .arg("--argc-eval")
        .arg(script_path())
        .arg("foo")
        .arg("hello")
        .assert()
        .stdout(predicate::str::contains("export ARGC_PARENT_ARGS=external"))
        .stdout(predicate::str::contains("bash"))
        .stdout(predicate::str::contains("external-foo.sh"))
        .stdout(predicate::str::contains("hello"))
        .success();
}

#[test]
fn external_external_subcommand_with_flags_generates_bash_call() {
    argc_bin()
        .arg("--argc-eval")
        .arg(script_path())
        .arg("bar")
        .arg("-v")
        .arg("hey")
        .assert()
        .stdout(predicate::str::contains("bash"))
        .stdout(predicate::str::contains("external-bar.sh"))
        .stdout(predicate::str::contains("-v"))
        .stdout(predicate::str::contains("hey"))
        .success();
}

#[test]
fn compgen_include_external_subcommands() {
    argc_bin()
        .arg("--argc-compgen")
        .arg("generic")
        .arg(script_path())
        .arg("external")
        .arg("")
        .assert()
        .stdout(predicate::str::contains("builtin"))
        .stdout(predicate::str::contains("foo"))
        .stdout(predicate::str::contains("bar"))
        .stdout(predicate::str::contains("help"))
        .success();
}

#[test]
fn compgen_show_external_subcommands_after_partial() {
    argc_bin()
        .arg("--argc-compgen")
        .arg("generic")
        .arg(script_path())
        .arg("external")
        .arg("f")
        .assert()
        .stdout(predicate::str::contains("foo"))
        .success();
}

#[test]
fn compgen_show_external_subcommand_flags() {
    argc_bin()
        .arg("--argc-compgen")
        .arg("generic")
        .arg(script_path())
        .arg("external")
        .arg("bar")
        .arg("--")
        .assert()
        .stdout(predicate::str::contains("verbose"))
        .success();
}

#[test]
fn invalid_subcommand_error_includes_external() {
    argc_bin()
        .arg("--argc-eval")
        .arg(script_path())
        .arg("abc")
        .assert()
        .stdout(predicate::str::contains(
            "subcommands: builtin, b, bar, foo",
        ))
        .success();
}
