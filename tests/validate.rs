use crate::*;

#[test]
fn help_version() {
    let script = r###"
# @describe Test argc
# @version    1.0.0
# @author     nobody <nobody@example.com>
"###;
    snapshot_multi!(
        script,
        vec![
            vec!["prog", "help"],
            vec!["prog", "--help"],
            vec!["prog", "-help"],
            vec!["prog", "-h"],
            vec!["prog", "--version"],
            vec!["prog", "-version"],
            vec!["prog", "-V"],
        ]
    );
}

#[test]
fn help_version_shadow() {
    let script = r###"
# @describe Test argc
# @version    1.0.0

# @flag -h --host
# @flag -V --verbose
"###;
    snapshot_multi!(script, vec![vec!["prog", "-h"], vec!["prog", "-V"],]);
}

#[test]
fn arg_help_subcmd() {
    snapshot!(SCRIPT_ARGS, &["prog", "help", "cmdd"]);
}

#[test]
fn arg_invalid_subcmd() {
    let script = r###"
# @cmd
cmda() { :; }
# @cmd
cmdb() { :; }
"###;
    snapshot!(script, &["prog", "foo"]);
}

#[test]
fn arg_missing() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmdd"]);
}

#[test]
fn arg_choice() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmdh", "val"]);
}

#[test]
fn arg_choice_fn() {
    snapshot!(
        &locate_script("args.sh"),
        SCRIPT_ARGS,
        &["prog", "cmdj", "val"]
    );
}

#[test]
fn arg_choice_fn_pass() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmdj", "val"]);
}

#[test]
fn arg_choice_fn_skip() {
    snapshot!(
        &locate_script("args.sh"),
        SCRIPT_ARGS,
        &["prog", "cmdk", "abc"]
    );
}

#[test]
fn arg_choice_multi() {
    snapshot!(
        &locate_script("args.sh"),
        SCRIPT_ARGS,
        &["prog", "cmdl", "abc", "val"]
    );
}

#[test]
fn arg_unknown() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmdb", "v1", "v2"]);
}

#[test]
fn flag_with_value() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "-a=3"]);
}

#[test]
fn flag_not_multiple() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "-a", "-a"]);
}

#[test]
fn option_unknown() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "--unknown"]);
}

#[test]
fn option_not_multiple() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "-e", "-e"]);
}

#[test]
fn option_mismatch_values() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "-o", "file1"]);
}

#[test]
fn option_choice() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "--ca", "val"]);
}

#[test]
fn option_choice_fn() {
    snapshot!(
        &locate_script("options.sh"),
        SCRIPT_OPTIONS,
        &["prog", "cmda", "--cc", "val"]
    );
}

#[test]
fn option_choice_fn_pass() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "--cc", "val"]);
}

#[test]
fn option_choice_fn_skip() {
    snapshot!(
        &locate_script("options.sh"),
        SCRIPT_OPTIONS,
        &["prog", "cmda", "--cd", "val"]
    );
}

#[test]
fn option_choice_multi() {
    snapshot!(
        &locate_script("options.sh"),
        SCRIPT_OPTIONS,
        &["prog", "cmda", "--ce", "abc", "val"]
    );
}

#[test]
fn option_missing() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmdb"]);
}

#[test]
fn param_missing() {
    let script = r###"
# @option --ao!
# @arg val!
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn param_missing_parent() {
    let script = r###"
# @option --ao!
# @cmd
foo() { :; }
"###;
    snapshot!(script, &["prog", "foo"]);
}

#[test]
fn empty_choices() {
    let script = r###"
# @arg val[`_choice_fn`]
_choice_fn() { :; }
"###;
    snapshot!(CREATE, script, &["prog", "foo"]);
}
