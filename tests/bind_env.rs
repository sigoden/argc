use rstest::rstest;

#[rstest]
fn bind_env_flags_help() {
    snapshot_bind_env!(args: ["flags", "-h"], envs: {});
}

#[rstest]
fn bind_env_flags() {
    snapshot_bind_env!(args: ["flags"], envs: {
        "FA1": "true",
        "FB2": "false",
        "FA": "true",
        "FC": "true",
        "FD": "true",
    });
}

#[rstest]
fn bind_env_flags_bool_err() {
    snapshot_bind_env!(args: ["flags"], envs: {
        "FA1": "v1",
    });
}

#[rstest]
fn bind_env_flags_bool_ok() {
    snapshot_bind_env!(args: ["flags", "--fa1"], envs: {
        "FA1": "v1",
    });
}

#[rstest]
fn bind_env_options_help() {
    snapshot_bind_env!(args: ["options", "-h"], envs: {});
}

#[rstest]
fn bind_env_options() {
    snapshot_bind_env!(args: ["options"], envs: {
        "OA1": "oa1",
        "OA2": "oa2",
        "OA": "oa3",
        "OB": "ob",
        "OC": "v1,v2",
        "ODA": "oda",
        "ODD": "odd",
        "OCA": "a",
        "OCC": "a",
        "OFA": "abc",
        "OFD": "abc,def",
        "OXA": "oxa",
    });
}

#[rstest]
fn bind_env_options_choice_err() {
    snapshot_bind_env!(args: ["options"], envs: {
        "OB": "ob",
        "OCA": "oca",
    });
}

#[rstest]
fn bind_env_options_choice_ok() {
    snapshot_bind_env!(args: ["options", "--oca", "a"], envs: {
        "OB": "ob",
        "OCA": "oca",
    });
}

#[rstest]
fn bind_env_options_choice_fn_err() {
    snapshot_bind_env!(args: ["options"], envs: {
        "OB": "ob",
        "OFA": "ofa",
    });
}

#[rstest]
fn bind_env_options_required_err() {
    snapshot_bind_env!(args: ["options"], envs: {});
}

#[rstest]
fn bind_env_arg1() {
    snapshot_bind_env!(args: ["cmd_arg1"], envs: {
        "VAL": "v1",
    });
}

#[rstest]
fn bind_env_arg2() {
    snapshot_bind_env!(args: ["cmd_arg2"], envs: {
        "VA": "v1",
    });
}

#[rstest]
fn bind_env_arg_choice_err() {
    snapshot_bind_env!(args: ["cmd_arg_with_choice"], envs: {
        "VAL": "v1",
    });
}

#[rstest]
fn bind_env_arg_choice_fn_err() {
    snapshot_bind_env!(args: ["cmd_arg_with_choice_fn"], envs: {
        "VAL": "v1",
    });
}

#[rstest]
fn bind_env_multi_arg_with_choice_fn_and_comma_sep() {
    snapshot_bind_env!(args: ["cmd_multi_arg_with_choice_fn_and_comma_sep"], envs: {
        "VAL": "abc,def",
    });
}

#[rstest]
fn bind_env_cmd_three_required_args() {
    snapshot_bind_env!(args: ["cmd_three_required_args"], envs: {
        "VAL1": "v1",
        "VAL2": "v2",
        "VAL3": "v3",
    });
}

#[rstest]
fn bind_env_cmd_three_required_args_err() {
    snapshot_bind_env!(args: ["cmd_three_required_args"], envs: {
        "VAL1": "v1",
        "VAL2": "v2",
    });
}
