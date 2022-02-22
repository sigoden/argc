use insta::assert_snapshot;

#[test]
fn test_spec_help() {
    assert_argc!(include_str!("spec.sh"), &["spec", "-h"]);
}

#[test]
fn test_spec_cmd_prefered_help() {
    assert_argc!(include_str!("spec.sh"), &["spec", "cmd-prefered", "-h"]);
}

#[test]
fn test_spec_cmd_omitted_help() {
    assert_argc!(include_str!("spec.sh"), &["spec", "cmd-omitted", "-h"]);
}

#[test]
fn test_spec_cmd_option_names_help() {
    assert_argc!(include_str!("spec.sh"), &["spec", "cmd-option-names", "-h"]);
}

#[test]
fn test_spec_cmd_option_formats_help() {
    assert_argc!(
        include_str!("spec.sh"),
        &["spec", "cmd-option-formats", "-h"]
    );
}

#[test]
fn test_spec_cmd_option_quotes_help() {
    assert_argc!(
        include_str!("spec.sh"),
        &["spec", "cmd-option-quotes", "-h"]
    );
}

#[test]
fn test_spec_cmd_flag_formats_help() {
    assert_argc!(include_str!("spec.sh"), &["spec", "cmd-flag-formats", "-h"]);
}

#[test]
fn test_spec_cmd_positional_only_help() {
    assert_argc!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-only", "-h"]
    );
}

#[test]
fn test_spec_cmd_positional_requires_help() {
    assert_argc!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-requires", "-h"]
    );
}

#[test]
fn test_spec_cmd_prefered_exec() {
    assert_argc!(
        include_str!("spec.sh"),
        &["spec", "cmd-prefered", "-f", "-o", "A", "AB", "C D"]
    );
}

#[test]
fn test_spec_cmd_option_names_exec() {
    assert_argc!(
        include_str!("spec.sh"),
        &[
            "spec",
            "cmd-option-names",
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
        ]
    );
}
