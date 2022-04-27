#[test]
fn test_spec_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "-h"],);
}

#[test]
fn test_spec_cmd_preferred_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd_preferred", "-h"],);
}

#[test]
fn test_spec_cmd_omitted_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd_omitted", "-h"],);
}

#[test]
fn test_spec_cmd_option_names_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd_option_names", "-h"],);
}

#[test]
fn test_spec_cmd_option_formats_help() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_option_formats", "-h"],
    );
}

#[test]
fn test_spec_cmd_option_quotes_help() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_option_quotes", "-h"],
    );
}

#[test]
fn test_spec_cmd_flag_formats_help() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd_flag_formats", "-h"],);
}

#[test]
fn test_spec_cmd_positional_only_help() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_only", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_requires_help() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_requires", "-h"],
    );
}

#[test]
fn test_spec_cmd_preferred_exec() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_preferred", "-f", "-o", "A", "AB", "C D"],
    );
}

#[test]
fn test_spec_cmd_option_names_exec() {
    snapshot!(
        include_str!("spec.sh"),
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
        ],
    );
}

#[test]
fn test_spec_cmd_option_names_exec_eval() {
    snapshot!(
        include_str!("spec.sh"),
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
        ],
    );
}

#[test]
fn test_spec_cmd_positional_with_default() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_default", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_with_default_exec() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_default"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_choices", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_exec() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_choices", "a"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_exec_fail() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_choices", "x"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_default() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_choices_and_default", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_default_exec() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_choices_and_default"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_required() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_choices_and_required", "-h"],
    );
}

#[test]
fn test_spec_cmd_positional_with_choices_and_required_exec_fail() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_positional_with_choices_and_required"],
    );
}

#[test]
fn test_spec_cmd_alias() {
    snapshot!(include_str!("spec.sh"), &["spec", "alias"],);
}

#[test]
fn test_spec_cmd_without_any_arg() {
    snapshot!(include_str!("spec.sh"), &["spec", "cmd_without_any_arg"],);
}

#[test]
fn test_spec_cmd_without_any_arg_exec() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_without_any_arg", "foo", "bar"],
    );
}

#[test]
fn test_spec_cmd_without_any_arg_exec_eval() {
    snapshot!(
        include_str!("spec.sh"),
        &["spec", "cmd_without_any_arg", "foo", "bar"],
    );
}

#[test]
fn test_spec_cmd_with_hyphens() {
    snapshot!(
        include_str!("spec.sh"),
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
