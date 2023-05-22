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
            " ",
            " --",
            " -- ",
            " -f ",
            " --fc ",
            " -o ",
            " -o d1",
            " -o d1 ",
            " -o d1 d2",
            " -o d1 d2 ",
            " -d d1",
            " -d d1 ",
            " -d d1 d2",
            " -d d1 d2 ",
            " -d d1 d2 ",
            " v1",
            " v1 ",
            " v1 v2",
            " v1 v2 ",
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
            " ", " -", " --", " -a", " -a ", " -af", " -af ", " -ae", " -ae ", " -abe", " -abe ",
            " -s", " -sa", " -sa ",
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
            " ",
            " c",
            " cmda",
            " cmda ",
            " help ",
            " help c",
            " help cmda ",
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
            " cmd",
            " cmd ",
            " cmd s",
            " cmd suba",
            " cmd suba ",
            " cmd help  ",
            " cmd help s",
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
            "cmda ",
            "cmda v1",
            "cmda v1 ",
            "cmda v1 v2",
            "cmda v1 v2 ",
            "cmdb ",
            "cmdb v1",
            "cmdb v1 ",
            "cmdb v1 v2",
            "cmdb v1 v2 ",
            "cmdb v1 v2 v3",
            "cmdb v1 v2 v3 ",
            "cmdc ",
            "cmdc v1",
            "cmdc v1 ",
            "cmdc v1 v2",
            "cmdc v1 v2 ",
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

    snapshot_compgen!(script, vec![" --oa ", " --ob ", " ", " v1 ",]);
}

#[test]
fn option_multi_vals() {
    let script = r###"
# @option --oa* <DIR> <FILE>
"###;
    snapshot_compgen!(script, vec![" --oa ", " --oa bash ", " --oa bash cmd1 ",]);
}
