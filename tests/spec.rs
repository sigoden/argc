use crate::*;

#[test]
fn option_help() {
    let mut names = vec![];
    for c in 'a'..='z' {
        let name = format!("cmd{c}");
        if SCRIPT_OPTIONS.contains(&name) {
            names.push(name);
        }
    }
    let matrix: Vec<Vec<&str>> = names
        .iter()
        .map(|v| vec!["prog", v.as_str(), "-h"])
        .collect();
    snapshot_multi!(SCRIPT_OPTIONS, &matrix);
}

#[test]
fn option_eval() {
    snapshot_multi!(
        SCRIPT_OPTIONS,
        [
            vec!["prog", "cmda"],
            vec!["prog", "cmda", "-a"],
            vec!["prog", "cmda", "-f", "-f"],
            vec!["prog", "cmda", "-e", "e"],
            vec!["prog", "cmda", "--oa", "oa"],
            vec!["prog", "cmda", "--ob", "ob1", "--ob", "ob2"],
            vec!["prog", "cmda", "--oe", "ob1,ob2", "--oe", "ob3"],
            vec!["prog", "cmda", "-o", "ob1", "ob2"],
            vec!["prog", "cmda", "--cc", "abc"],
            vec!["prog", "cmda", "-soa", "soa"],
            vec!["prog", "cmdc"],
            vec!["prog", "cmdc", "--oe", "oe"],
            vec!["prog", "cmdc", "--of", "of"],
            vec!["prog", "cmdc", "--cb", "y"],
        ]
    );
}

#[test]
fn option_shorts() {
    snapshot_multi!(
        SCRIPT_OPTIONS,
        [
            vec!["prog", "cmda", "-af"],
            vec!["prog", "cmda", "-ae", "e"],
            vec!["prog", "cmda", "-afe", "e"],
            vec!["prog", "cmda", "-ao", "v1", "v2"],
        ]
    );
}

#[test]
fn arg_eval() {
    snapshot_multi!(
        SCRIPT_ARGS,
        [
            vec!["prog", "cmdb", "v1"],
            vec!["prog", "cmdc", "v1", "v2"],
            vec!["prog", "cmdf"],
            vec!["prog", "cmdf", "v1"],
            vec!["prog", "cmdg"],
            vec!["prog", "cmdh", "x"],
            vec!["prog", "cmdj", "abc"],
            vec!["prog", "cmdp", "v1", "v2"],
            vec!["prog", "cmdp", "v1", "v2", "v3"],
            vec!["prog", "cmdr", "v1", "v2", "v3"],
        ]
    );
}

#[test]
fn arg_subcmd_help() {
    let mut names = vec![];
    for c in 'a'..='z' {
        let name = format!("cmd{c}");
        if SCRIPT_ARGS.contains(&name) {
            names.push(name);
        }
    }
    let matrix: Vec<Vec<&str>> = names
        .iter()
        .map(|v| vec!["prog", v.as_str(), "-h"])
        .collect();
    snapshot_multi!(SCRIPT_ARGS, &matrix);
}

#[test]
fn arg_no_param() {
    snapshot_multi!(
        SCRIPT_ARGS,
        [
            vec!["prog", "cmda", "v1", "v2"],
            vec!["prog", "cmda", "--o1", "-o2", "-3"]
        ]
    );
}

#[test]
fn arg_no_option() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmdc", "--o1", "-o2", "-3"]);
}

#[test]
fn arg_two_multi() {
    snapshot_multi!(
        SCRIPT_ARGS,
        [
            vec!["prog", "cmdp", "a", "b", "c"],
            vec!["prog", "cmdp", "--", "a", "b", "c"],
            vec!["prog", "cmdp", "a", "--", "b", "c"],
            vec!["prog", "cmdp", "a", "b", "--", "c"],
            vec!["prog", "cmdp", "a", "b", "c", "--"],
        ]
    );
}

#[test]
fn dash_split() {
    let script = r###"
# @flag -f
# @option --oa
# @arg v1
# @arg v2*
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "v1", "-f", "--oa", "a"],
            vec!["prog", "v1", "-f", "--", "--oa", "a"],
        ]
    );
}

#[test]
fn undefine_positionals() {
    let script = r###"
# @option --oa
"###;
    snapshot!(script, &["prog", "--oa", "v1", "v2"]);
}

#[test]
fn same_option_positional() {
    let script = r###"
# @option --url
# @arg url
"###;
    snapshot!(script, &["prog", "-h"]);
}

#[test]
fn option_multiple() {
    let script = r###"
# @flag   -f --fc*
# @option -a --oa* <DIR>
# @option -b --ob <CMD> <DIR+>
# @option -c --oc <DIR+>
# @option -d --od <DIR> <FILE>
# @option -e --oe* <DIR+>
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "-h"],
            vec!["prog", "-f", "-f"],
            vec!["prog", "-a", "dir1", "dir2"],
            vec!["prog", "-a", "dir1", "-a", "dir2"],
            vec!["prog", "-b", "vim", "dir1", "dir2"],
            vec!["prog", "-c", "dir1", "dir2"],
            vec!["prog", "-d", "dir1", "file1", "file2"],
            vec!["prog", "-e", "dir1", "-e", "dir2", "dir3"],
        ]
    );
}

#[test]
fn option_to_variable() {
    let script = r###"
# @flag --flag-foo
# @option --option-foo*
# @option --option-bar
# @arg arg-foo
# @arg arg-bar*
"###;
    snapshot!(
        script,
        &[
            "prog",
            "--flag-foo",
            "--option-foo",
            "foo1",
            "--option-foo",
            "foo2",
            "--option-bar",
            "bar",
            "v1",
            "x1",
            "x2"
        ]
    );
}

#[test]
fn option_terminated() {
    let script = r###"
# @option --oa~ <SHELL> <SCRIPT> <ARGS>
# @option --ob
"###;
    snapshot!(script, &["prog", "--oa", "bash", "Argcfile.sh", "--ob"]);
}

#[test]
fn arg_terminated() {
    let script = r###"
# @cmd
# @arg args~
cmda() {
    :;
}
"###;
    snapshot!(script, &["prog", "cmda", "-h"]);
}

#[test]
fn option_prefixed() {
    let script = r###"
# @option -o-
# @option -D-*
"###;
    snapshot!(script, &["prog", "-D", "v1", "-Dv2=foo", "-o1"]);
}

#[test]
fn cmd_with_hyphen() {
    let script = r###"
# @cmd Run --foo
# @flag --fa
--foo() {
    :;
}

# @cmd Run bar
# @alias -B
# @flag --fa
bar() {
    :;
}
"###;
    snapshot_multi!(
        script,
        [vec!["prog", "--foo", "--fa"], vec!["prog", "-B", "--fa"]]
    );
}

#[test]
fn cmd_combine_shorts() {
    let script = r###"
# @meta combine-shorts
# @cmd
# @flag -B
# @flag -C
-A() {
    :;
}
"###;
    snapshot_multi!(script, [vec!["prog", "-A"], vec!["prog", "-AB"]]);
}

#[test]
fn name_with_special_chars() {
    let script = r###"
# @flag --fa:foo
# @flag --fa.bar
# @flag --fa_baz
"###;
    snapshot_multi!(script, [vec!["prog", "--fa:foo", "--fa.bar", "--fa_baz"]]);
}

#[test]
fn inherit_flag_options() {
    let script = r###"
# @meta inherit-flag-options
# @flag --oa
# @option --ob[a|b]  desc 1

# @cmd
cmda() {
    :;
}

# @cmd
# @option --ob[x|y]  desc 2
cmdb() {
    :;
}
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "cmda", "--ob", "a"],
            vec!["prog", "cmdb", "--ob", "x"]
        ]
    );
}

#[test]
fn symbol() {
    let script = r###"
# @meta symbol +toolchain
# @option --oa
"###;
    snapshot_multi!(script, [vec!["prog", "+nightly"]]);
}

#[test]
fn plus_sign() {
    let script = r###"
# @flag +a
# @option +fb
# @option +c +fc*
# @option +d -fd*
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "+a", "+fb", "fb", "+c", "fc1", "+fc", "fc2"],
            vec!["prog", "+d", "fd1", "-fd", "fd2"],
        ]
    );
}

#[test]
fn zero_or_one() {
    let script = r###"
# @option --oa <VALUE?>
"###;
    snapshot_multi!(script, [vec!["prog", "--oa"], vec!["prog", "--oa", "v1"]]);
}
