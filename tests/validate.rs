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
# @flag -V --verify
"###;
    snapshot_multi!(script, vec![vec!["prog", "-h"], vec!["prog", "-V"],]);
}

#[test]
fn help_version_exist() {
    let script = r###"
# @flag -h --help
# @flag -V --version
"###;
    snapshot_multi!(script, vec![vec!["prog", "-h"]]);
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
    snapshot!(SCRIPT_ARGS, &["prog", "cmdj", "val"]);
}

#[test]
fn arg_choice_fn_pass() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmdj", "val"], None, None);
}

#[test]
fn arg_choice_fn_skip() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmdk", "abc"]);
}

#[test]
fn arg_choice_multi() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmdl", "abc", "val"]);
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
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "--cc", "val"]);
}

#[test]
fn option_choice_fn_pass() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "--cc", "val"], None, None);
}

#[test]
fn option_choice_fn_skip() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "--cd", "val"]);
}

#[test]
fn option_choice_multi() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "cmda", "--ce", "abc", "val"]);
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
# @option --oa[`_choice_fn1`]
_choice_fn1() {
	:;
 }
"###;
    snapshot_multi!(script, vec![vec!["prog", "--oa", "foo"],]);
}

#[test]
fn choice_access_vars() {
    let script = r###"
# @flag --fa
# @arg val[`_choice_fn`]
_choice_fn() {
	if [[ $argc_fa == 1 ]]; then
		echo abc
	else
		echo def
	fi
 }
"###;
    snapshot_multi!(
        script,
        vec![vec!["prog", "--fa", "foo"], vec!["prog", "foo"],]
    );
}

#[test]
fn choice_slash() {
    let script = r###"
# @cmd
# @arg foo
# @arg bar[`_choice_fn`]
cmd() {
    echo $1
}
_choice_fn() {
    echo $1
}
"###;
    snapshot_multi!(script, vec![vec!["prog", "cmd", "a\\b", "a\\b"],]);
}

#[test]
fn cmd_name_sanitize() {
    let script = r###"
# @cmd
cat_() {
    echo run cat_
}

# @cmd
do_() {
    echo run do_
}

# @cmd
do_::foo() {
    echo run do_::foo
}

# @cmd
do_::bar() {
    echo run do_::bar
}

# @cmd
foo_bar() {
    echo run foo_bar
}
"###;
    snapshot_multi!(
        script,
        vec![
            vec!["prog", "--help"],
            vec!["prog", "cat", "--help"],
            vec!["prog", "cat"],
            vec!["prog", "do", "--help"],
            vec!["prog", "do"],
            vec!["prog", "do", "foo"],
            vec!["prog", "foo-bar"],
        ]
    );
}
