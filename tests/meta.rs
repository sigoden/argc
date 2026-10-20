#[test]
fn dotenv() {
    let script = r###"
# @meta dotenv
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn dotenv_custom_path() {
    let script = r###"
# @meta dotenv .env.local
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn dotenv_with_env_var_in_path() {
    let tmpdir = crate::fixtures::tmpdir();
    let dotenv_dir = tmpdir.path().join("tmp");
    std::fs::create_dir_all(&dotenv_dir).unwrap();
    std::fs::write(dotenv_dir.join(".env.2"), "ARGC_TEST_API_TOKEN=abc\n").unwrap();
    let script = r###"
# @meta dotenv "$ARGC_TEST_DIR/tmp/.env.2"
# @env ARGC_TEST_API_TOKEN!
main() {
    echo "token=$ARGC_TEST_API_TOKEN"
}
"###;
    let (script_path, _, script_file) =
        crate::fixtures::create_argc_script(script, "dotenv-env-var.sh");
    let dir = tmpdir.path().to_str().unwrap().to_string();
    let envs = vec![("ARGC_TEST_DIR", dir.as_str())];
    let output = crate::fixtures::run_script(&script_path, &[], &envs);
    assert!(output.contains("token=abc"), "output: {output}");
    script_file.close().unwrap();
}

#[test]
fn dotenv_with_multiple_paths() {
    let tmpdir = crate::fixtures::tmpdir();
    std::fs::write(tmpdir.path().join(".env"), "ARGC_TEST_DOTENV_A=1\n").unwrap();
    std::fs::write(tmpdir.path().join(".env.local"), "ARGC_TEST_DOTENV_B=2\n").unwrap();
    let script = r###"
# @meta dotenv "$ARGC_TEST_DIR/.env" "$ARGC_TEST_DIR/.env.local"
# @env ARGC_TEST_DOTENV_A!
# @env ARGC_TEST_DOTENV_B!
main() {
    echo "A=$ARGC_TEST_DOTENV_A B=$ARGC_TEST_DOTENV_B"
}
"###;
    let (script_path, _, script_file) =
        crate::fixtures::create_argc_script(script, "dotenv-multiple.sh");
    let dir = tmpdir.path().to_str().unwrap().to_string();
    let envs = vec![("ARGC_TEST_DIR", dir.as_str())];
    let output = crate::fixtures::run_script(&script_path, &[], &envs);
    assert!(output.contains("A=1 B=2"), "eval output: {output}");

    let build_dir = crate::fixtures::tmpdir();
    let build_script_path = crate::fixtures::build_script(&build_dir, script, "prog");
    let build_output = crate::fixtures::run_script(&build_script_path, &[], &envs);
    assert!(
        build_output.contains("A=1 B=2"),
        "build output: {build_output}"
    );
    script_file.close().unwrap();
}

#[test]
fn binname() {
    let script = r###"
# @meta binname test-binname
"###;
    snapshot!(script, &["prog", "-h"]);
}
