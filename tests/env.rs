use rstest::rstest;

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
