use crate::fixtures::{argc_bin, get_path_env_var, locate_script, tmpdir};
use argc::Runtime;

fn has_git_bash() -> bool {
    argc::NativeRuntime.bash_path().is_some()
}

#[test]
fn eval_restores_msys_no_pathconv() {
    let path = locate_script("examples/args");
    let output = argc_bin()
        .arg("--argc-eval")
        .arg(&path)
        .arg("cmd")
        .env("ARGC_MSYS_NO_PATHCONV_RESTORE", "1")
        .env("ARGC_MSYS_NO_PATHCONV", "0")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("export MSYS_NO_PATHCONV=\"$ARGC_MSYS_NO_PATHCONV\""));
    assert!(stdout.contains("export ARGC_MSYS_NO_PATHCONV\n"));
    assert!(stdout.contains(
        "--argc-eval|--argc-parallel) MSYS_NO_PATHCONV=1 ARGC_MSYS_NO_PATHCONV_RESTORE=1 command argc \"$@\" ;;"
    ));
    assert!(stdout.contains("--argc-*) command argc \"$@\" ;;"));
    assert!(stdout
        .contains("*) MSYS_NO_PATHCONV=1 ARGC_MSYS_NO_PATHCONV_RESTORE=1 command argc \"$@\" ;;"));
    assert!(stdout.contains("export -f argc"));
}

#[test]
fn eval_unsets_msys_no_pathconv_when_original_unset() {
    let path = locate_script("examples/args");
    let output = argc_bin()
        .arg("--argc-eval")
        .arg(&path)
        .arg("cmd")
        .env("ARGC_MSYS_NO_PATHCONV_RESTORE", "1")
        .env_remove("ARGC_MSYS_NO_PATHCONV")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unset MSYS_NO_PATHCONV"));
    assert!(stdout.contains("export -f argc"));
}

#[test]
fn eval_without_restore_marker_keeps_output_unchanged() {
    let path = locate_script("examples/args");
    let output = argc_bin()
        .arg("--argc-eval")
        .arg(&path)
        .arg("cmd")
        .env_remove("ARGC_MSYS_NO_PATHCONV_RESTORE")
        .env_remove("ARGC_MSYS_NO_PATHCONV")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("MSYS_NO_PATHCONV"));
}

#[test]
fn shim_preserves_raw_args_and_restores_env() {
    if !has_git_bash() {
        return;
    }
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    argc_bin()
        .current_dir(tmpdir.path())
        .env("PATH", &path_env_var)
        .arg("--argc-create")
        .arg("mycli")
        .arg("run")
        .assert()
        .success();
    std::fs::write(
        tmpdir.path().join("mycli"),
        r#"#!/usr/bin/env bash
# @cmd
run() {
    echo "arg=[$1]"
    echo "env=[${MSYS_NO_PATHCONV-unset}]"
}
eval "$(argc --argc-eval "$0" "$@")"
"#,
    )
    .unwrap();

    let shim = tmpdir.path().join("mycli.cmd");
    let run = |msys: Option<&str>| {
        let mut command = std::process::Command::new("cmd");
        command
            .arg("/C")
            .arg(&shim)
            .arg("run")
            .arg("/foo")
            .current_dir(tmpdir.path())
            .env("PATH", &path_env_var)
            .env_remove("ARGC_MSYS_NO_PATHCONV")
            .env_remove("ARGC_MSYS_NO_PATHCONV_RESTORE");
        match msys {
            Some(value) => {
                command.env("MSYS_NO_PATHCONV", value);
            }
            None => {
                command.env_remove("MSYS_NO_PATHCONV");
            }
        }
        command.output().unwrap()
    };

    for (msys, expected_env) in [(None, "unset"), (Some("0"), "0")] {
        let output = run(msys);
        assert!(output.status.success(), "{output:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("arg=[/foo]"), "{stdout}");
        assert!(
            stdout.contains(&format!("env=[{expected_env}]")),
            "{stdout}"
        );
    }
}

#[test]
fn shim_preserves_raw_args_in_nested_script() {
    if !has_git_bash() {
        return;
    }
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    argc_bin()
        .current_dir(tmpdir.path())
        .env("PATH", &path_env_var)
        .arg("--argc-create")
        .arg("parent")
        .arg("run")
        .assert()
        .success();
    std::fs::write(
        tmpdir.path().join("parent"),
        r#"#!/usr/bin/env bash
# @cmd
run() {
    echo "parent-arg=[$1]"
    echo "parent-env=[${MSYS_NO_PATHCONV-unset}]"
    bash "$(dirname "$0")/child.sh" run "$1"
}
eval "$(argc --argc-eval "$0" "$@")"
"#,
    )
    .unwrap();
    std::fs::write(
        tmpdir.path().join("child.sh"),
        r#"#!/usr/bin/env bash
# @cmd
run() {
    echo "child-arg=[$1]"
    echo "child-env=[${MSYS_NO_PATHCONV-unset}]"
}
eval "$(argc --argc-eval "$0" "$@")"
"#,
    )
    .unwrap();

    let output = std::process::Command::new("cmd")
        .arg("/C")
        .arg(tmpdir.path().join("parent.cmd"))
        .arg("run")
        .arg("/foo")
        .current_dir(tmpdir.path())
        .env("PATH", &path_env_var)
        .env_remove("MSYS_NO_PATHCONV")
        .env_remove("ARGC_MSYS_NO_PATHCONV")
        .env_remove("ARGC_MSYS_NO_PATHCONV_RESTORE")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("parent-arg=[/foo]"), "{stdout}");
    assert!(stdout.contains("parent-env=[unset]"), "{stdout}");
    assert!(stdout.contains("child-arg=[/foo]"), "{stdout}");
    assert!(stdout.contains("child-env=[unset]"), "{stdout}");
}

#[test]
fn runner_preserves_raw_args() {
    if !has_git_bash() {
        return;
    }
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    std::fs::write(
        tmpdir.path().join("Argcfile.sh"),
        r#"#!/usr/bin/env bash
# @cmd
run() {
    echo "arg=[$1]"
    echo "env=[${MSYS_NO_PATHCONV-unset}]"
}
eval "$(argc --argc-eval "$0" "$@")"
"#,
    )
    .unwrap();

    let output = argc_bin()
        .current_dir(tmpdir.path())
        .env("PATH", &path_env_var)
        .env_remove("MSYS_NO_PATHCONV")
        .env_remove("ARGC_MSYS_NO_PATHCONV")
        .env_remove("ARGC_MSYS_NO_PATHCONV_RESTORE")
        .args(["run", "/foo"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("arg=[/foo]"), "{stdout}");
    assert!(stdout.contains("env=[unset]"), "{stdout}");
}

#[test]
fn eval_reads_posix_script_path_with_pathconv_disabled() {
    use argc::Runtime;
    let Some(bash) = argc::NativeRuntime.bash_path() else {
        return;
    };
    let argc = assert_cmd::cargo::cargo_bin!();
    let script = format!("/tmp/argc-posix-probe-{}.sh", std::process::id());
    let command = format!(
        "cat > '{script}' <<'EOF'\n#!/usr/bin/env bash\n# @cmd\nrun() {{ echo ok; }}\nEOF\nMSYS_NO_PATHCONV=1 '{}' --argc-eval '{script}' run\nrc=$?\nrm -f '{script}'\nexit $rc",
        argc.display()
    );
    let output = std::process::Command::new(bash)
        .args(["--noprofile", "--norc", "-c", &command])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("argc__fn=run"), "{stdout}");
}

#[test]
fn eval_posix_argcfile_uses_normalized_script_dir() {
    let Some(bash) = argc::NativeRuntime.bash_path() else {
        return;
    };
    let tmpdir = tmpdir();
    let script = tmpdir.path().join("Argcfile.sh");
    std::fs::write(
        &script,
        r#"#!/usr/bin/env bash
# @cmd
run() { printf 'pwd=%s\n' "$PWD"; }
eval "$(argc --argc-eval "$0" "$@")"
"#,
    )
    .unwrap();

    let script_arg = to_posix_path(&script);
    let output = std::process::Command::new(bash)
        .args(["--noprofile", "--norc", &script_arg, "run"])
        .env("PATH", get_path_env_var())
        .env("MSYS_NO_PATHCONV", "1")
        .env_remove("ARGC_MSYS_NO_PATHCONV")
        .env_remove("ARGC_MSYS_NO_PATHCONV_RESTORE")
        .env_remove("ARGC_PWD")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let dir_name = tmpdir.path().file_name().unwrap().to_string_lossy();
    assert!(stdout.contains("pwd="), "{stdout}");
    assert!(stdout.contains(dir_name.as_ref()), "{stdout}");
}

#[test]
fn wrapper_preserves_path_conversion_for_other_subcommands() {
    let Some(bash) = argc::NativeRuntime.bash_path() else {
        return;
    };
    let tmpdir = tmpdir();
    let nested = tmpdir.path().join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let posix_probe = to_posix_path(&nested.join("probe"));
    let script = tmpdir.path().join("probe-script");
    std::fs::write(
        &script,
        format!(
            r#"#!/usr/bin/env bash
# @cmd
run() {{
    argc --argc-create '{posix_probe}' inner >/dev/null 2>&1 || true
    if [ -e '{posix_probe}' ]; then echo posix-created; fi
}}
eval "$(argc --argc-eval "$0" "$@")"
"#
        ),
    )
    .unwrap();

    let output = std::process::Command::new(&bash)
        .args(["--noprofile", "--norc"])
        .arg(&script)
        .arg("run")
        .env("PATH", get_path_env_var())
        .env("ARGC_MSYS_NO_PATHCONV_RESTORE", "1")
        .env_remove("MSYS_NO_PATHCONV")
        .env_remove("ARGC_MSYS_NO_PATHCONV")
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("posix-created"), "{stdout}");
}

fn to_posix_path(path: &std::path::Path) -> String {
    let path = path.display().to_string().replace('\\', "/");
    let bytes = path.as_bytes();
    if bytes.len() >= 3 && bytes[1] == b':' {
        format!("/{}{}", path[..1].to_lowercase(), &path[2..])
    } else {
        path
    }
}

#[test]
fn parallel_preserves_raw_args() {
    if !has_git_bash() {
        return;
    }
    let tmpdir = tmpdir();
    let path_env_var = get_path_env_var();
    let script = tmpdir.path().join("parallel-probe.sh");
    std::fs::write(
        &script,
        r#"#!/usr/bin/env bash
# @cmd
run() {
    echo "arg=[$1]"
    echo "env=[${MSYS_NO_PATHCONV-unset}]"
}
eval "$(argc --argc-eval "$0" "$@")"
"#,
    )
    .unwrap();

    let output = argc_bin()
        .arg("--argc-parallel")
        .arg(&script)
        .args(["run", "/foo"])
        .env("PATH", &path_env_var)
        .env_remove("MSYS_NO_PATHCONV")
        .env_remove("ARGC_MSYS_NO_PATHCONV")
        .env_remove("ARGC_MSYS_NO_PATHCONV_RESTORE")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("arg=[/foo]"), "{stdout}");
    assert!(stdout.contains("env=[unset]"), "{stdout}");
}
