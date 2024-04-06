use crate::*;

#[test]
fn help_version() {
    let script = r###"
# @describe Test argc
# @meta version 1.0.0
# @meta author nobody <nobody@example.com>
"###;
    snapshot_multi!(
        script,
        [
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
fn help_version_legacy() {
    let script = r###"
# @describe Test argc
# @version 1.0.0
# @author nobody <nobody@example.com>
"###;
    snapshot_multi!(
        script,
        [
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
# @meta version 1.0.0

# @flag -h --host
# @flag -V --verify
"###;
    snapshot_multi!(script, [vec!["prog", "-h"], vec!["prog", "-V"],]);
}

#[test]
fn help_version_exist() {
    let script = r###"
# @describe Test argc
# @meta version 1.0.0

# @flag -h --help
# @flag -V --version
"###;
    snapshot_multi!(script, [vec!["prog", "-h"], vec!["prog", "-V"]]);
}

#[test]
fn help_notations() {
    let script = r###"
# @option -target <name>            <arch><sub>-<os>-<abi> see the targets command
# @option -n <num>                  <num> volumes for input, '0' to prompt interactively
# @option --merge <path1> <path2> <base> <result>  Perform a three-way merge by providing paths for two modified versions of a file, the common origin of both modified versions and the output file to save merge results.
"###;
    snapshot_multi!(script, [vec!["prog", "-h"]]);
}

#[test]
fn version_missing() {
    let script = r###"
# @cmd
cmd() { :; }
"###;
    snapshot_multi!(
        script,
        [vec!["prog", "--version"], vec!["prog", "cmd", "--version"]]
    );
}

#[test]
fn arg_help_subcmd() {
    snapshot!(SCRIPT_ARGS, &["prog", "help", "cmd_required_multi_arg"]);
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
    snapshot!(SCRIPT_ARGS, &["prog", "cmd_required_multi_arg"]);
}

#[test]
fn arg_choice() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmd_arg_with_choices", "val"]);
}

#[test]
fn arg_choice_fn() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmd_arg_with_choice_fn", "val"]);
}

#[test]
fn arg_choice_fn_pass() {
    snapshot!(
        SCRIPT_ARGS,
        &["prog", "cmd_arg_with_choice_fn", "val"],
        None,
        None
    );
}

#[test]
fn arg_choice_fn_skip() {
    snapshot!(
        SCRIPT_ARGS,
        &["prog", "cmd_arg_with_choice_fn_and_skip_check", "abc"]
    );
}

#[test]
fn arg_choice_multi() {
    snapshot!(
        SCRIPT_ARGS,
        &["prog", "cmd_multi_arg_with_choice_fn", "abc", "val"]
    );
}

#[test]
fn arg_unknown() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmd_arg", "v1", "v2"]);
}

#[test]
fn flag_with_value() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test1", "-a=3"]);
}

#[test]
fn flag_not_multiple() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test1", "-a", "-a"]);
}

#[test]
fn option_unknown() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test1", "--unknown"]);
}

#[test]
fn option_not_multiple() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test1", "-e", "-e"]);
}

#[test]
fn option_mismatch_values() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test1", "-o", "file1"]);
}

#[test]
fn option_choice() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test1", "--ca", "val"]);
}

#[test]
fn option_choice_fn() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test1", "--cc", "val"]);
}

#[test]
fn option_choice_fn_pass() {
    snapshot!(
        SCRIPT_OPTIONS,
        &["prog", "test1", "--cc", "val"],
        None,
        None
    );
}

#[test]
fn option_choice_fn_skip() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test1", "--cd", "val"]);
}

#[test]
fn option_choice_multi() {
    snapshot!(
        SCRIPT_OPTIONS,
        &["prog", "test1", "--ce", "abc", "--ce", "val"]
    );
}

#[test]
fn option_missing() {
    snapshot!(SCRIPT_OPTIONS, &["prog", "test2"]);
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
    snapshot_multi!(script, [vec!["prog", "--oa", "foo"],]);
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
    snapshot_multi!(script, [vec!["prog", "--fa", "foo"], vec!["prog", "foo"],]);
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
    snapshot_multi!(script, [vec!["prog", "cmd", "a\\b", "a\\b"],]);
}

#[test]
fn delimiter() {
    let script = r###"
# @cmd
# @option --oa*,[`_choice_fn`]
# @arg val*,[`_choice_fn`]
cmd() { :; }

_choice_fn() {
    echo -e "abc\ndef\nijk"
}
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "cmd", "--oa", "abc,def"],
            vec!["prog", "cmd", "abc,def"]
        ]
    );
}

#[test]
fn cmd_name_sanitize() {
    let script = r###"
# @cmd
cat_() { :; }

# @cmd
do_() { :; }

# @cmd
do_::foo() { :; }

# @cmd
do_::bar() { :; }
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "--help"],
            vec!["prog", "cat", "--help"],
            vec!["prog", "cat"],
            vec!["prog", "do", "--help"],
            vec!["prog", "do"],
            vec!["prog", "do", "foo"],
        ]
    );
}
