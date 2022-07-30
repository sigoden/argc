use super::SPEC_SCRIPT;

#[test]
fn test_compgen() {
    snapshot_compgen!(SPEC_SCRIPT, &["prog"]);
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
