use rstest::rstest;

#[rstest]
fn bind_env_flags_help() {
    snapshot_bind_env!(args: ["flags", "-h"], envs: {});
}

#[rstest]
fn bind_env_flags() {
    snapshot_bind_env!(args: ["flags"], envs: {
        "BIND_ENVS_FA1": "true",
        "FB2": "false",
        "FA": "true",
        "BIND_ENVS_FC": "true",
        "BIND_ENVS_FD": "true",
    });
}

#[rstest]
fn bind_env_flags_bool_err() {
    snapshot_bind_env!(args: ["flags"], envs: {
        "BIND_ENVS_FA1": "v1",
    });
}

#[rstest]
fn bind_env_flags_bool_ok() {
    snapshot_bind_env!(args: ["flags", "--fa1"], envs: {
        "BIND_ENVS_FA1": "v1",
    });
}

#[rstest]
fn bind_env_options_help() {
    snapshot_bind_env!(args: ["options", "-h"], envs: {});
}

#[rstest]
fn bind_env_options() {
    snapshot_bind_env!(args: ["options"], envs: {
        "BIND_ENVS_OA1": "oa1",
        "BIND_ENVS_OA2": "oa2",
        "OA": "oa3",
        "OB": "ob",
        "BIND_ENVS_OC": "v1,v2",
        "BIND_ENVS_ODA": "oda",
        "BIND_ENVS_ODB": "odd",
        "BIND_ENVS_OCA": "a",
        "BIND_ENVS_OCC": "a",
        "BIND_ENVS_OFA": "abc",
        "BIND_ENVS_OFD": "abc,def",
        "BIND_ENVS_OXA": "oxa",
    });
}

#[rstest]
fn bind_env_options_choice_err() {
    snapshot_bind_env!(args: ["options"], envs: {
        "OB": "ob",
        "BIND_ENVS_OCA": "oca",
    });
}

#[rstest]
fn bind_env_options_choice_ok() {
    snapshot_bind_env!(args: ["options", "--oca", "a"], envs: {
        "OB": "ob",
        "BIND_ENVS_OCA": "oca",
    });
}

#[rstest]
fn bind_env_options_choice_fn_err() {
    snapshot_bind_env!(args: ["options"], envs: {
        "OB": "ob",
        "BIND_ENVS_OFA": "ofa",
    });
}

#[rstest]
fn bind_env_options_required_err() {
    snapshot_bind_env!(args: ["options"], envs: {});
}

#[rstest]
fn bind_env_arg1() {
    snapshot_bind_env!(args: ["cmd_arg1"], envs: {
        "BIND_ENVS_VAL": "v1",
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
        "BIND_ENVS_VAL": "v1",
    });
}

#[rstest]
fn bind_env_arg_choice_fn_err() {
    snapshot_bind_env!(args: ["cmd_arg_with_choice_fn"], envs: {
        "BIND_ENVS_VAL": "v1",
    });
}

#[rstest]
fn bind_env_multi_arg_with_choice_fn_and_comma_sep() {
    snapshot_bind_env!(args: ["cmd_multi_arg_with_choice_fn_and_comma_sep"], envs: {
        "BIND_ENVS_VAL": "abc,def",
    });
}

#[rstest]
fn bind_env_cmd_three_required_args() {
    snapshot_bind_env!(args: ["cmd_three_required_args"], envs: {
        "BIND_ENVS_VAL1": "v1",
        "BIND_ENVS_VAL2": "v2",
        "BIND_ENVS_VAL3": "v3",
    });
}

#[rstest]
fn bind_env_cmd_three_required_args_err() {
    snapshot_bind_env!(args: ["cmd_three_required_args"], envs: {
        "BIND_ENVS_VAL1": "v1",
        "BIND_ENVS_VAL2": "v2",
    });
}

#[rstest]
fn bind_env_with_notation() {
    snapshot_bind_env!(args: ["cmd_for_notation"], envs: {
        "BIND_ENVS_OA": "oa",
        "BIND_ENVS_VAL": "v1",
    });
}
#[rstest]
fn bind_env_with_notation_help() {
    snapshot_bind_env!(args: ["cmd_for_notation", "-h"], envs: {});
}

#[rstest]
fn bind_env_inherit_flag_options_before_subcmd() {
    let script = r###"
# @meta inherit-flag-options
# @option --opt $ENV <PATH>  Option

# @cmd
cmd() {
    echo "argc_opt=${argc_opt:?}"
}

eval "$(argc --argc-eval "$0" "$@")"
"###;
    let (script_path, _, script_file) =
        crate::fixtures::create_argc_script(script, "inherit-bind-env.sh");
    let args: Vec<String> = vec!["--opt".into(), "arg".into(), "cmd".into()];
    let envs = vec![("ENV", "env")];
    let output = crate::fixtures::run_script(&script_path, &args, &envs);
    assert!(
        output.contains("argc_opt=arg"),
        "expected explicit option before subcommand, got: {output}"
    );
    script_file.close().unwrap();
}

#[rstest]
fn bind_env_inherit_flag_options_before_and_after_subcmd() {
    let script = r###"
# @meta inherit-flag-options
# @option --opt $ENV <PATH>  Option

# @cmd
cmd() {
    echo "argc_opt=${argc_opt:?}"
}

eval "$(argc --argc-eval "$0" "$@")"
"###;
    let (script_path, _, script_file) =
        crate::fixtures::create_argc_script(script, "inherit-bind-env2.sh");
    let args: Vec<String> = vec![
        "--opt".into(),
        "arg1".into(),
        "cmd".into(),
        "--opt".into(),
        "arg2".into(),
    ];
    let envs = vec![("ENV", "env")];
    let output = crate::fixtures::run_script(&script_path, &args, &envs);
    assert!(
        output.contains("argc_opt=arg2"),
        "expected last explicit option after subcommand to win, got: {output}"
    );
    script_file.close().unwrap();
}
