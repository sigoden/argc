#[test]
fn test_compgen() {
    snapshot_compgen!("");
}

#[test]
fn test_compgen_help() {
    snapshot_compgen!("help ");
}

#[test]
fn test_compgen_help2() {
    snapshot_compgen!("help cmd");
}

#[test]
fn test_compgen_subcommand() {
    snapshot_compgen!("cmd_option_names ");
}

#[test]
fn test_compgen_option_choices() {
    snapshot_compgen!("cmd_option_names --opt7 ");
}

#[test]
fn test_compgen_option_choices2() {
    snapshot_compgen!("cmd_option_names --opt7 a ");
}

#[test]
fn test_compgen_positional() {
    snapshot_compgen!("cmd_positional_requires ");
}

#[test]
fn test_compgen_positional_arg() {
    snapshot_compgen!("cmd_positional_requires arg1 ");
}

#[test]
fn test_compgen_positional_arg2() {
    snapshot_compgen!("cmd_positional_requires arg1 arg2 ");
}

#[test]
fn test_compgen_positional_choices() {
    snapshot_compgen!("cmd_positional_with_choices ");
}

#[test]
fn test_compgen_help_tag() {
    snapshot_compgen!("cmd_nested_command");
}

#[test]
fn test_compgen_help_tag2() {
    snapshot_compgen!("cmd_nested_command help");
}

#[test]
fn test_compgen_nested_command() {
    snapshot_compgen!("cmd_nested_command ");
}

#[test]
fn test_compgen_nested_command_subcommand() {
    snapshot_compgen!("cmd_nested_command foo ");
}

#[test]
fn test_compgen_nested_command_subcommand2() {
    snapshot_compgen!("cmd_nested_command fo");
}

#[test]
fn test_compgen_multiple_notation() {
    snapshot_compgen!("cmd_option_notations --opt2");
}

#[test]
fn test_compgen_multiple_notation2() {
    snapshot_compgen!("cmd_option_notations --opt2 foo ");
}

#[test]
fn test_compgen_multiple_notation3() {
    snapshot_compgen!("cmd_option_notations --opt2 foo bar ");
}

#[test]
fn test_compgen_unknown_option() {
    snapshot_compgen!("cmd_omitted --unknown foo --flag1 ");
}

#[test]
fn test_compgen_unknown_option2() {
    snapshot_compgen!("cmd_omitted --unknown foo ");
}

#[test]
fn test_compgen_many_positionals() {
    snapshot_compgen!("cmd_positional_many foo ");
}

#[test]
fn test_compgen_cmd_no_args() {
    snapshot_compgen!("cmd_without_any_arg ");
}

#[test]
fn test_compgen_combine_shorts() {
    snapshot_compgen!("cmd_combine_shorts -a");
}

#[test]
fn test_compgen_combine_shorts1() {
    snapshot_compgen!("cmd_combine_shorts -ab");
}

#[test]
fn test_compgen_combine_shorts3() {
    snapshot_compgen!("cmd_combine_shorts -ac");
}
