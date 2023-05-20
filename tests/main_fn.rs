use crate::*;

#[test]
fn with_main() {
    let script = r###"
# @arg val
main() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn no_main() {
    let script = r###"
# @arg val
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn subcmd_main() {
    let script = r###"
# @cmd
cmd() { :; }
main() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn subcmd_no_main() {
    let script = r###"
# @cmd
cmd() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn nested_subcmd_main() {
    let script = r###"
# @cmd
cmd() { :; }
cmd::main() { :; }
# @cmd
cmd::foo() { :; }
"###;
    snapshot!(script, &["prog", "cmd"]);
}

#[test]
fn nested_subcmd_no_main() {
    let script = r###"
# @cmd
cmd() { :; }
# @cmd
cmd::foo() { :; }
"###;
    snapshot!(script, &["prog", "cmd"]);
}
