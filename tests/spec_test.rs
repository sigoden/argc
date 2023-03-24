use super::SPEC_SCRIPT;

#[test]
fn test_spec_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "-h"],);
}

#[test]
fn test_spec_cmd_preferred_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_preferred", "-h"],);
}

#[test]
fn test_spec_cmd_omitted_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_omitted", "-h"],);
}

#[test]
fn test_spec_cmd_option_names_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_option_names", "-h"],);
}

#[test]
fn test_spec_cmd_option_formats_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_option_formats", "-h"],);
}

#[test]
fn test_spec_cmd_option_quotes_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_option_quotes", "-h"],);
}

#[test]
fn test_spec_cmd_flag_formats_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_flag_formats", "-h"],);
}

#[test]
fn test_spec_cmd_flag_formats_count() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "cmd_flag_formats", "--foo5", "--foo5", "--foo5"],
    );
}

#[test]
fn test_spec_cmd_positional_only_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_positional_only", "-h"],);
}

#[test]
fn test_spec_cmd_positional_requires_help() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_positional_requires", "-h"],);
}

#[test]
fn test_spec_cmd_preferred_exec() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "cmd_preferred", "-f", "-o", "A", "AB", "C D"],
    );
}

#[test]
fn test_spec_cmd_option_names_exec() {
    snapshot!(
        SPEC_SCRIPT,
        &[
            "spec",
            "cmd_option_names",
            "--opt2",
            "value2",
            "--opt3",
            "value3_0,value3_1",
            "--opt4",
            "value4_0",
            "--opt4",
            "value4_1",
            "--opt6",
            "a",
            "--opt8",
            "a",
            "--op11",
            "a1"
        ],
    );
}

#[test]
fn test_spec_cmd_option_names_exec_eval() {
    snapshot!(
        SPEC_SCRIPT,
        &[
            "spec",
            "cmd_option_names",
            "--opt2",
            "value2",
            "--opt3",
            "value3_0,value3_1",
            "--opt4",
            "value4_0",
            "--opt4",
            "value4_1",
            "--opt6",
            "a",
            "--opt8",
            "a",
            "--op11",
            "a1"
        ],
    );
}

#[test]
fn test_spec_cmd_positional_with_default() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_positional_with_default", "-h"],);
}

#[test]
fn test_spec_cmd_positional_with_default_exec() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_positional_with_default"],);
}

#[test]
fn test_spec_cmd_positional_with_choices() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_positional_with_choices", "-h"],);
}

#[test]
fn test_spec_cmd_positional_with_choices_exec() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_positional_with_choices", "a"],);
}

#[test]
fn test_spec_cmd_positional_with_choices_exec_fail() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_positional_with_choices", "x"],);
}

#[test]
fn test_spec_cmd_positional_with_choices_and_default() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "cmd_positional_with_choices_and_default", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_default_exec() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "cmd_positional_with_choices_and_default"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_required() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "cmd_positional_with_choices_and_required", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_required_exec_fail() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "cmd_positional_with_choices_and_required"],
    );
}

#[test]
fn test_spec_cmd_alias() {
    snapshot!(SPEC_SCRIPT, &["spec", "alias"],);
}

#[test]
fn test_spec_cmd_without_any_arg() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_without_any_arg"],);
}

#[test]
fn test_spec_cmd_without_any_arg_exec() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_without_any_arg", "foo", "bar"],);
}

#[test]
fn test_spec_cmd_without_any_arg_exec_eval() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_without_any_arg", "foo", "bar"],);
}

#[test]
fn test_spec_cmd_with_hyphens() {
    snapshot!(
        SPEC_SCRIPT,
        &[
            "spec",
            "cmd_with_hyphens",
            "foo",
            "--hyphen-flag",
            "--hyphen-option",
            "bar"
        ],
    );
}

#[test]
fn test_spec_fn_bars() {
    snapshot!(SPEC_SCRIPT, &["spec", "_fn_bars",],);
}

#[test]
fn test_spec_fn_args() {
    snapshot!(SPEC_SCRIPT, &["spec", "_fn_args","-z 5 -y=6 -y8 --msg cool -7 --here=there -xvf file.tgz -qrs=1234 -n -555 one two three -abc+5 -c-6 -- four -z 0"],);
}

#[test]
fn test_spec_nested_command() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_nested_command",],);
}

#[test]
fn test_spec_nested_command_exec() {
    snapshot!(
        SPEC_SCRIPT,
        &["spec", "cmd_nested_command", "foo", "--opt1", "abc"],
    );
}

#[test]
fn test_spec_nested_command2() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_nested_command2",],);
}

#[test]
fn test_no_long_flags() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_no_long_flags", "-h"],);
}

#[test]
fn test_no_long_options() {
    snapshot!(SPEC_SCRIPT, &["spec", "cmd_no_long_options", "-h"],);
}
