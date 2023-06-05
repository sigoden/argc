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
            "prog ",
            "prog --",
            "prog -- ",
            "prog -f ",
            "prog --fc ",
            "prog -o ",
            "prog -o d1",
            "prog -o d1 ",
            "prog -o d1 d2",
            "prog -o d1 d2 ",
            "prog -d d1",
            "prog -d d1 ",
            "prog -d d1 d2",
            "prog -d d1 d2 ",
            "prog v1",
            "prog v1 ",
            "prog v1 v2",
            "prog v1 v2 ",
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
            "prog ",
            "prog -",
            "prog --",
            "prog -a",
            "prog -a ",
            "prog -af",
            "prog -af ",
            "prog -ae",
            "prog -ae ",
            "prog -abe",
            "prog -abe ",
            "prog -s",
            "prog -sa",
            "prog -sa ",
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
            "prog ",
            "prog c",
            "prog cmda",
            "prog cmda ",
            "prog help ",
            "prog help c",
            "prog help cmda ",
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
            "prog cmd",
            "prog cmd ",
            "prog cmd s",
            "prog cmd suba",
            "prog cmd suba ",
            "prog cmd help ",
            "prog cmd help s",
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
            "prog cmda ",
            "prog cmda v1",
            "prog cmda v1 ",
            "prog cmda v1 v2",
            "prog cmda v1 v2 ",
            "prog cmdb ",
            "prog cmdb v1",
            "prog cmdb v1 ",
            "prog cmdb v1 v2",
            "prog cmdb v1 v2 ",
            "prog cmdb v1 v2 v3",
            "prog cmdb v1 v2 v3 ",
            "prog cmdc ",
            "prog cmdc v1",
            "prog cmdc v1 ",
            "prog cmdc v1 v2",
            "prog cmdc v1 v2 ",
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
            "prog --oa ",
            "prog --oa=",
            "prog --ob ",
            "prog ",
            "prog v1 ",
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

    snapshot_compgen!(script, vec!["prog --oa ", "prog --oa="]);
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

    snapshot_compgen!(script, vec!["prog argc", "prog argc ", "prog argc argc"]);
}

#[test]
fn option_multi_vals() {
    let script = r###"
# @option --oa* <DIR> <FILE>
"###;
    snapshot_compgen!(
        script,
        vec!["prog --oa ", "prog --oa bash ", "prog --oa bash cmd1 ",]
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
    snapshot_compgen!(script, vec!["prog ", "prog cmda ",]);
}

#[test]
fn no_param() {
    let script = r###"
# @cmd
cmd() { :; }
"###;
    snapshot_compgen!(script, vec!["prog cmd ",]);
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
    snapshot_compgen!(script, vec!["prog cmda ", "prog cmdb "]);
}

#[test]
fn one_combine_shorts() {
    let script = r###"
# @flag -a
# @flag -b
"###;
    snapshot_compgen!(script, vec!["prog -a"]);
}
