use crate::*;

#[test]
fn hook_without_subcmd() {
    let script = r###"
_argc_before() { :; }
_argc_after() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn hook_with_main() {
    let script = r###"
_argc_before() { :; }
_argc_after() { :; }
main() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn hook_with_subcmd() {
    let script = r###"
_argc_before() { :; }
_argc_after() { :; }
# @cmd
cmd() { :; }
"###;
    snapshot!(script, &["prog", "cmd"]);
}

#[test]
fn hook_only_before() {
    let script = r###"
_argc_before() { :; }
main() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn hook_only_after() {
    let script = r###"
_argc_after() { :; }
main() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn hook_param_fn() {
    let script = r###"
_argc_before() { :; }
_argc_after() { :; }
_choice_fn() { :; }
"###;
    snapshot!(script, &["prog", "___internal___", "_choice_fn"]);
}
