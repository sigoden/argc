#[test]
fn test_no_help_subcommand() {
    let script = r###"
# @cmd
cmd() { :; }
    "###;
    snapshot!(script, &["prog"],);
}

#[test]
fn test_add_help_subcommand() {
    let script = r###"
# @help Print help information
# @cmd
cmd() { :; }
    "###;
    snapshot!(script, &["prog"],);
}
