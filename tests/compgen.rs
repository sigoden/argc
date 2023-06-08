use crate::*;

#[test]
fn multiple() {
    let script = r###"
# @flag   -f --fc*
# @option -o --oa* <DIR>
# @option -d --od <DIR> <FILE>
# @arg var* <FILE>
"###;

    snapshot_compgen!(
        script,
        vec![
            vec!["prog", ""],
            vec!["prog", "--"],
            vec!["prog", "--", ""],
            vec!["prog", "-f", ""],
            vec!["prog", "--fc", ""],
            vec!["prog", "-o", ""],
            vec!["prog", "-o", "d1"],
            vec!["prog", "-o", "d1", ""],
            vec!["prog", "-o", "d1", "d2"],
            vec!["prog", "-o", "d1", "d2", ""],
            vec!["prog", "-d", "d1"],
            vec!["prog", "-d", "d1", ""],
            vec!["prog", "-d", "d1", "d2"],
            vec!["prog", "-d", "d1", "d2", ""],
            vec!["prog", "v1"],
            vec!["prog", "v1", ""],
            vec!["prog", "v1", "v2"],
            vec!["prog", "v1", "v2", ""],
        ]
    );
}

#[test]
fn shorts() {
    const SCRIPT: &str = r###"
# @flag   -a
# @flag   -b --fb
# @flag   -f --fc*
# @flag      -sa
# @option -e <FILE>
# @option -p --oa*
"###;

    snapshot_compgen!(
        SCRIPT,
        vec![
            vec!["prog", ""],
            vec!["prog", "-"],
            vec!["prog", "--"],
            vec!["prog", "-a"],
            vec!["prog", "-a", ""],
            vec!["prog", "-af"],
            vec!["prog", "-af", ""],
            vec!["prog", "-ae"],
            vec!["prog", "-ae", ""],
            vec!["prog", "-abe"],
            vec!["prog", "-abe", ""],
            vec!["prog", "-s"],
            vec!["prog", "-sa"],
            vec!["prog", "-sa", ""],
        ]
    );
}

#[test]
fn subcmds() {
    const SCRIPT: &str = r###"
# @arg file
# @cmd
cmda() { :; }
# @cmd
cmdb() { :; }
"###;

    snapshot_compgen!(
        SCRIPT,
        vec![
            vec!["prog", ""],
            vec!["prog", "c"],
            vec!["prog", "cmda"],
            vec!["prog", "cmda", ""],
            vec!["prog", "help", ""],
            vec!["prog", "help", "c"],
            vec!["prog", "help", "cmda", ""],
        ]
    );
}

#[test]
fn nested_subcmds() {
    const SCRIPT: &str = r###"
# @arg file
# @cmd
cmd() { :; }
# @cmd
cmd::suba() { :; }
# @cmd
cmd::subb() { :; }
"###;

    snapshot_compgen!(
        SCRIPT,
        vec![
            vec!["prog", "cmd"],
            vec!["prog", "cmd", ""],
            vec!["prog", "cmd", "s"],
            vec!["prog", "cmd", "suba"],
            vec!["prog", "cmd", "suba", ""],
            vec!["prog", "cmd", "help", ""],
            vec!["prog", "cmd", "help", "s"],
        ]
    );
}

#[test]
fn positionals() {
    let script = r###"
# @cmd
# @arg dir
# @arg file*
cmda() { :; }

# @cmd
# @arg dir1
# @arg dir2
# @arg dir3
cmdb() { :; }

# @cmd
# @arg dir*
# @arg file*
cmdc() { :; }
"###;

    snapshot_compgen!(
        script,
        vec![
            vec!["prog", "cmda", ""],
            vec!["prog", "cmda", "v1"],
            vec!["prog", "cmda", "v1", ""],
            vec!["prog", "cmda", "v1", "v2"],
            vec!["prog", "cmda", "v1", "v2", ""],
            vec!["prog", "cmdb", ""],
            vec!["prog", "cmdb", "v1"],
            vec!["prog", "cmdb", "v1", ""],
            vec!["prog", "cmdb", "v1", "v2"],
            vec!["prog", "cmdb", "v1", "v2", ""],
            vec!["prog", "cmdb", "v1", "v2", "v3"],
            vec!["prog", "cmdb", "v1", "v2", "v3", ""],
            vec!["prog", "cmdc", ""],
            vec!["prog", "cmdc", "v1"],
            vec!["prog", "cmdc", "v1", ""],
            vec!["prog", "cmdc", "v1", "v2"],
            vec!["prog", "cmdc", "v1", "v2", ""],
        ]
    );
}

#[test]
fn choice() {
    let script = r###"
# @option --oa[`_choice_fn`]
# @option --ob[x|y|z]
# @arg v1[x|y|z]
# @arg v2[`_choice_fn`]
_choice_fn() {
	echo -e "abc\ndef\nghi"
}
"###;

    snapshot_compgen!(
        script,
        vec![
            vec!["prog", "--oa", ""],
            vec!["prog", "--oa="],
            vec!["prog", "--oa=a"],
            vec!["prog", "--oa", "=a"],
            vec!["prog", "--ob", ""],
            vec!["prog", ""],
            vec!["prog", "v1", ""],
            vec!["prog", "'--oa="],
            vec!["prog", "'--oa=a"],
            vec!["prog", "\"--oa="],
            vec!["prog", "\"--oa=a"],
        ]
    );
}

#[test]
fn choice_multi() {
    let script = r###"
# @option --oa*[`_choice_fn`]
_choice_fn() {
	echo -e "abc\ndef\nghi"
}
"###;

    snapshot_compgen!(
        script,
        vec![vec!["prog", "--oa", ""], vec!["prog", "--oa="],]
    );
}

#[test]
fn choice_check_vars() {
    let script = r###"
# @arg foo[`_choice_fn`]
# @arg bar[`_choice_fn`]
_choice_fn() {
    ( set -o posix ; set ) | grep argc_
}
"###;

    snapshot_compgen!(
        script,
        vec![
            vec!["prog", "argc"],
            vec!["prog", "argc", ""],
            vec!["prog", "argc", "argc"],
        ]
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
    snapshot_compgen!(script, vec![vec!["prog", "cmd", "a\\b", ""],]);
}

#[test]
fn option_multi_vals() {
    let script = r###"
# @option --oa* <DIR> <FILE>
"###;
    snapshot_compgen!(
        script,
        vec![
            vec!["prog", "--oa", ""],
            vec!["prog", "--oa", "bash", ""],
            vec!["prog", "--oa", "bash", "cmd1", ""],
        ]
    );
}

#[test]
fn multiline_doc() {
    let script = r###"
# @cmd cmd-line1
# cmd-line2
# @option --foo option-line1
# option-line2
# @arg bar bar-line1
# bar-line2
cmda() { :; }

# @cmd line
cmdb() { :; }
"###;
    snapshot_compgen!(script, vec![vec!["prog", ""], vec!["prog", "cmda", ""],]);
}

#[test]
fn no_param() {
    let script = r###"
# @cmd
cmd() { :; }
"###;
    snapshot_compgen!(script, vec![vec!["prog", "cmd", ""],]);
}

#[test]
fn special_arg_name() {
    let script = r###"
# @cmd
# @arg arg
cmda() { :; }

# @cmd
# @arg any
cmdb() { :; }
"###;
    snapshot_compgen!(
        script,
        vec![vec!["prog", "cmda", ""], vec!["prog", "cmdb", ""],]
    );
}

#[test]
fn one_combine_shorts() {
    let script = r###"
# @flag -a
# @flag -b
"###;
    snapshot_compgen!(script, vec![vec!["prog", "-a"],]);
}

#[test]
fn no_comp_subcmds() {
    let script = r###"
# @cmd
cmda() { :; }

# @cmd
cmdb() { :; }
"###;
    snapshot_compgen!(
        script,
        vec![
            vec!["prog", ""],
            vec!["prog", "cmdx", ""],
            vec!["prog", "cmdx", "cmd"]
        ]
    );
}

#[test]
fn no_space() {
    let script = r###"
# @option --oa*[`_choice_fn`]
_choice_fn() {
	echo -e "abc"
	echo -e "def\0"
	echo -e "ghk\thello world"
}
"###;

    snapshot_compgen_shells!(script, vec!["prog", "--oa", ""]);
}
