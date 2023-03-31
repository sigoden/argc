use super::SPEC_SCRIPT;

#[test]
fn test_param_fn_empty() {
    snapshot!(SPEC_SCRIPT, &["spec", "_fn_args"],);
}

#[test]
fn test_param_fn_space() {
    snapshot!(SPEC_SCRIPT, &["spec", "_fn_args", " "],);
}

#[test]
fn test_param_fn_args() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "_fn_args", "cmd_preferred -f -o 4 abc"],
    );
}

#[test]
fn test_param_fn_args_dup_flag() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "_fn_args", "cmd_preferred -f -o 4 -f"],
    );
}

#[test]
fn test_param_fn_args_dup_option() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "_fn_args", "cmd_preferred -f -o 4 -o 5"],
    );
}

#[test]
fn test_param_fn_args_dup_dashdash1() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "_fn_args", "cmd_preferred -f -o 4 -- abc"],
    );
}

#[test]
fn test_param_fn_args_dup_dashdash2() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "_fn_args", "cmd_preferred -f -o 4 abc --"],
    );
}

#[test]
fn test_param_fn_args_dup_dashdash3() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "_fn_args", "cmd_preferred -f -o 4 abc -- def"],
    );
}

#[test]
fn test_param_fn_args_incomplete_option() {
    snapshot!(SPEC_SCRIPT, &["spec", "_fn_args", "cmd_preferred -f -o"],);
}

#[test]
fn test_param_fn_args_nontmatch_subcommand() {
    snapshot!(SPEC_SCRIPT, &["spec", "_fn_args", "cmd_prefer"],);
}

#[test]
fn test_param_fn_args_unknown_option() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "_fn_args", "cmd_preferred -f -o 4 -x"],
    );
}

#[test]
fn test_param_fn_bars() {
    snapshot!(SPEC_SCRIPT, &["spec", "_fn_bars",],);
}
