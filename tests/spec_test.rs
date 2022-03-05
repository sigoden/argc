#[test]
fn test_spec_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "-h"],);
}

#[test]
fn test_spec_cmd_preferred_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd-preferred", "-h"],);
}

#[test]
fn test_spec_cmd_omitted_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd-omitted", "-h"],);
}

#[test]
fn test_spec_cmd_option_names_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd-option-names", "-h"],);
}

#[test]
fn test_spec_cmd_option_formats_help() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-option-formats", "-h"],
    );
}

#[test]
fn test_spec_cmd_option_quotes_help() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-option-quotes", "-h"],
    );
}

#[test]
fn test_spec_cmd_flag_formats_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd-flag-formats", "-h"],);
}

#[test]
fn test_spec_cmd_positional_only_help() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-only", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_requires_help() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-requires", "-h"],
    );
}

#[test]
fn test_spec_cmd_preferred_exec() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-preferred", "-f", "-o", "A", "AB", "C D"],
    );
}

#[test]
fn test_spec_cmd_option_names_exec() {
    snapshot!(
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
        ],
    );
}

#[test]
fn test_spec_cmd_option_names_exec_eval() {
    snapshot!(
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
        ],
        eval: true,
    );
}

#[test]
fn test_spec_cmd_positional_with_choices() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-with-choices", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_exec() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-with-choices", "a"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_exec_fail() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-with-choices", "x"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_default() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-with-choices-and-default", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_default_exec() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd-positional-with-choices-and-default"],
    );
}

#[test]
fn test_spec_cmd_without_any_arg() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd-without-any-arg"],);
}
