use crate::*;

#[test]
fn hook_without_subcmd() {
    let script = r###"
_argc_init() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn hook_with_main() {
    let script = r###"
_argc_init() { :; }
main() { :; }
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn hook_with_subcmd() {
    let script = r###"
_argc_init() { :; }
# @cmd
cmd() { :; }
"###;
    snapshot!(script, &["prog", "cmd"]);
}

#[test]
fn hook_param_fn() {
    let script = r###"
_argc_init() { :; }
_choice_fn() { :; }
"###;
    snapshot!(script, &["prog", "___internal___", "_choice_fn"]);
}
