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
fn with_main2() {
    let script = r###"
# @option --foo
# @arg val
main() { :; }
"###;
    snapshot_multi!(script, [vec!["prog"], vec!["prog", "abc", "--foo", "123"]]);
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
fn nested_subcmd_main2() {
    let script = r###"
# @cmd
# @option --foo
# @arg val
cmd() { :; }
cmd::main() { :; }
# @cmd
cmd::foo() { :; }
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "cmd"],
            vec!["prog", "cmd", "abc", "--foo", "123"]
        ]
    );
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

#[test]
fn global_with_arg() {
    let script = r###"
# @arg val
# @cmd
cmd() { :; }
main() { :; }
"###;
    snapshot!(script, &["prog", "abc"]);
}

#[test]
fn global_without_arg() {
    let script = r###"
# @cmd
cmd() { :; }
main() { :; }
"###;
    snapshot!(script, &["prog", "abc"]);
}
