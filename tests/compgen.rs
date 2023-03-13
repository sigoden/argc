use super::SPEC_SCRIPT;

const HELP_TAG_SCRIPT: &str = r#"
# @help Print help information

# @cmd
foo() { :; }

# @cmd
bar() { :; }
"#;

#[test]
fn test_compgen() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog"]);
}

#[test]
fn test_compgen_help() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog", "help"]);
}

#[test]
fn test_compgen_subcommand() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog", "cmd_option_names"]);
}

#[test]
fn test_compgen_option_choices() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog", "cmd_option_names", "--opt7"]);
}

#[test]
fn test_compgen_positional() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog", "cmd_positional_requires"]);
}

#[test]
fn test_compgen_positional_arg() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog", "cmd_positional_requires", "arg1"]);
}

#[test]
fn test_compgen_positional_arg2() {
    snapshot_compgen!(
        SPEC_SCRIPT,
        &["prog", "cmd_positional_requires", "arg1", "arg2"]
    );
}

#[test]
fn test_compgen_positional_choices() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog", "cmd_positional_with_choices"]);
}

#[test]
fn test_compgen_help_tag() {
    snapshot_compgen!(HELP_TAG_SCRIPT, &["prog"]);
}

#[test]
fn test_compgen_help_tag2() {
    snapshot_compgen!(HELP_TAG_SCRIPT, &["prog", "help"]);
}

#[test]
fn test_compgen_chocie_fn() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog", "cmd_option_names", "--op11"]);
}
