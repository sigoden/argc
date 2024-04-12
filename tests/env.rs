use rstest::rstest;

#[rstest]
fn env_help() {
    snapshot_meta_env!(["-h"], {});
}

#[rstest]
fn env_help_subcmd() {
    snapshot_meta_env!(["run", "-h"], {});
}

#[rstest]
fn env_missing() {
    snapshot_meta_env!([], {});
}

#[rstest]
fn env_choice() {
    snapshot_meta_env!([], {"TEST_EB": "1", "TEST_ECA": "val"});
}

#[rstest]
fn env_choice_fn() {
    snapshot_meta_env!([], {"TEST_EB": "1", "TEST_EFA": "val"});
}

#[rstest]
fn env_run() {
    snapshot_meta_env!([], {"TEST_EB": "1"});
}
