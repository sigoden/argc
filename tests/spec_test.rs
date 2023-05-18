#[test]
fn test_spec_help() {
    snapshot_spec!(&["spec", "-h"]);
}

#[test]
fn test_spec_cmd_preferred_help() {
    snapshot_spec!(&["spec", "cmd_preferred", "-h"]);
}

#[test]
fn test_spec_cmd_omitted_help() {
    snapshot_spec!(&["spec", "cmd_omitted", "-h"]);
}

#[test]
fn test_spec_cmd_option_names_help() {
    snapshot_spec!(&["spec", "cmd_option_names", "-h"]);
}

#[test]
fn test_spec_cmd_option_names_choices_fn() {
    snapshot_spec!(&[
        "spec",
        "cmd_option_names",
        "--opt2",
        "v2",
        "--opt4",
        "v4",
        "--opt8",
        "v8",
        "--op11",
        "xyz"
    ]);
}

#[test]
fn test_spec_cmd_option_formats_help() {
    snapshot_spec!(&["spec", "cmd_option_formats", "-h"]);
}

#[test]
fn test_spec_cmd_option_quotes_help() {
    snapshot_spec!(&["spec", "cmd_option_quotes", "-h"]);
}

#[test]
fn test_spec_cmd_flag_formats_help() {
    snapshot_spec!(&["spec", "cmd_flag_formats", "-h"]);
}

#[test]
fn test_spec_cmd_flag_formats_count() {
    snapshot_spec!(&["spec", "cmd_flag_formats", "--foo5", "--foo5", "--foo5"]);
}

#[test]
fn test_spec_cmd_positional_only_help() {
    snapshot_spec!(&["spec", "cmd_positional_only", "-h"]);
}

#[test]
fn test_spec_cmd_positional_requires_help() {
    snapshot_spec!(&["spec", "cmd_positional_requires", "-h"]);
}

#[test]
fn test_spec_cmd_preferred_exec() {
    snapshot_spec!(&["spec", "cmd_preferred", "-f", "-o", "A", "AB", "C D"]);
}

#[test]
fn test_spec_cmd_option_names_exec() {
    snapshot_spec!(&[
        "spec",
        "cmd_option_names",
        "--opt2",
        "value2",
        "--opt3",
        "value3_0",
        "--opt3",
        "value3_1",
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
    ]);
}

#[test]
fn test_spec_cmd_option_names_exec_eval() {
    snapshot_spec!(&[
        "spec",
        "cmd_option_names",
        "--opt2",
        "value2",
        "--opt3",
        "value3_0",
        "--opt3",
        "value3_1",
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
    ]);
}

#[test]
fn test_spec_cmd_positional_with_default() {
    snapshot_spec!(&["spec", "cmd_positional_with_default", "-h"]);
}

#[test]
fn test_spec_cmd_positional_with_default_exec() {
    snapshot_spec!(&["spec", "cmd_positional_with_default"]);
}

#[test]
fn test_spec_cmd_positional_with_choices() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices", "-h"]);
}

#[test]
fn test_spec_cmd_positional_with_choices_exec() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices", "a"]);
}

#[test]
fn test_spec_cmd_positional_with_choices_exec_fail() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices", "x"]);
}

#[test]
fn test_spec_cmd_positional_with_choices_and_default() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices_and_default", "-h"]);
}

#[test]
fn test_spec_cmd_positional_with_choices_and_default_exec() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices_and_default"]);
}

#[test]
fn test_spec_cmd_positional_with_choices_and_required() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices_and_required", "-h"]);
}

#[test]
fn test_spec_cmd_positional_with_choices_and_required_exec_fail() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices_and_required"]);
}

#[test]
fn test_spec_cmd_alias() {
    snapshot_spec!(&["spec", "alias"]);
}

#[test]
fn test_spec_cmd_without_any_arg() {
    snapshot_spec!(&["spec", "cmd_without_any_arg"]);
}

#[test]
fn test_spec_cmd_without_any_arg2() {
    snapshot_spec!(&["spec", "cmd_without_any_arg", "--opt2", "foo", "bar"]);
}

#[test]
fn test_spec_cmd_without_any_arg_exec() {
    snapshot_spec!(&["spec", "cmd_without_any_arg", "foo", "bar"]);
}

#[test]
fn test_spec_cmd_without_any_arg_exec_eval() {
    snapshot_spec!(&["spec", "cmd_without_any_arg", "foo", "bar"]);
}

#[test]
fn test_spec_cmd_with_hyphens() {
    snapshot_spec!(&[
        "spec",
        "cmd_with_hyphens",
        "foo",
        "--hyphen-flag",
        "--hyphen-option",
        "bar"
    ]);
}

#[test]
fn test_spec_nested_command() {
    snapshot_spec!(&["spec", "cmd_nested_command",]);
}

#[test]
fn test_spec_nested_command_exec() {
    snapshot_spec!(&["spec", "cmd_nested_command", "foo", "--opt1", "abc"]);
}

#[test]
fn test_spec_nested_command2() {
    snapshot_spec!(&["spec", "cmd_nested_command2",]);
}

#[test]
fn test_spec_nested_command3() {
    snapshot_spec!(&["spec", "cmd_nested_command2", "foo"]);
}

#[test]
fn test_spec_cmd_no_long_flags() {
    snapshot_spec!(&["spec", "cmd_no_long_flags", "-h"]);
}

#[test]
fn test_spec_cmd_no_long_options() {
    snapshot_spec!(&["spec", "cmd_no_long_options", "-h"]);
}

#[test]
fn test_spec_cmd_option_notations_help() {
    snapshot_spec!(&["spec", "cmd_option_notations", "-h"]);
}

#[test]
fn test_spec_cmd_option_notations() {
    snapshot_spec!(&["spec", "cmd_option_notations", "--opt2"]);
}

#[test]
fn test_spec_cmd_option_notations1() {
    snapshot_spec!(&["spec", "cmd_option_notations", "--opt2", "foo"]);
}

#[test]
fn test_spec_cmd_option_notations2() {
    snapshot_spec!(&["spec", "cmd_option_notations", "--opt2", "foo", "bar"]);
}

#[test]
fn test_spec_cmd_notation_values() {
    snapshot_spec!(&["spec", "cmd_notation_values", "-h"]);
}

#[test]
fn test_spec_help_command() {
    snapshot_spec!(&["spec", "help"]);
}

#[test]
fn test_spec_help_command2() {
    snapshot_spec!(&["spec", "help", "cmd_preferred"]);
}

#[test]
fn test_spec_help_command3() {
    snapshot_spec!(&["spec", "help", "cmd_prefered"]);
}

#[test]
fn test_spec_cmd_combine_shorts() {
    snapshot_spec!(&["spec", "cmd_combine_shorts", "-ac", "yes"]);
}

#[test]
fn test_spec_cmd_single_dash_help() {
    snapshot_spec!(&["spec", "cmd_single_dash", "-help"]);
}

#[test]
fn test_spec_cmd_single_dash() {
    snapshot_spec!(&["spec", "cmd_single_dash", "-flag1", "-opt1", "abc"]);
}

#[test]
fn test_spec_cmd_positional_with_choices_fn() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices_fn", "xyz"]);
}

#[test]
fn test_spec_cmd_positional_with_choices_fn2() {
    snapshot_spec!(&["spec", "cmd_positional_with_choices_fn2", "xyz"]);
}

#[test]
fn test_spec_cmd_two_multiple_positionals() {
    snapshot_spec!(&["spec", "cmd_two_multiple_positionals", "abc", "def", "cjk"]);
}

#[test]
fn test_spec_cmd_two_multiple_positionals2() {
    snapshot_spec!(&[
        "spec",
        "cmd_two_multiple_positionals",
        "--",
        "abc",
        "def",
        "cjk"
    ]);
}

#[test]
fn test_spec_cmd_two_multiple_positionals3() {
    snapshot_spec!(&[
        "spec",
        "cmd_two_multiple_positionals",
        "abc",
        "--",
        "def",
        "cjk"
    ]);
}
