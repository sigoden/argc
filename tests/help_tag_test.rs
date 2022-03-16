#[test]
fn test_no_help_tag() {
    let script = r###"
# @cmd
cmd() { :; }
    "###;
    plain!(script, &["prog"], stderr: r#"prog 

USAGE:
    prog <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    cmd     
    help    Print this message or the help of the given subcommand(s)
"#,);
}

#[test]
fn test_no_help_subcommand() {
    let script = r###"
# @help false
# @cmd
cmd() { :; }
    "###;
    plain!(script, &["prog"], stderr: r#"prog 

USAGE:
    prog <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    cmd    
"#,);
}

#[test]
fn test_custom_help_subcommand_about() {
    let script = r###"
# @help Print help information
# @cmd
cmd() { :; }
    "###;
    plain!(script, &["prog"], stderr: r#"prog 

USAGE:
    prog <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    cmd     
    help    Print help information
"#,);
}
