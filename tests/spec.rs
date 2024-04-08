use crate::*;

#[test]
fn option_help() {
    let names: Vec<String> = SCRIPT_OPTIONS
        .lines()
        .filter(|v| v.contains("() {") && !v.starts_with('_'))
        .map(|v| v.split_once('(').unwrap().0.to_string())
        .collect();
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
            vec!["prog", "test1"],
            vec!["prog", "test1", "-a"],
            vec!["prog", "test1", "-f", "-f"],
            vec!["prog", "test1", "-e", "e"],
            vec!["prog", "test1", "--oa", "oa"],
            vec!["prog", "test1", "--ob", "ob1", "--ob", "ob2"],
            vec!["prog", "test1", "--oe", "ob1,ob2", "--oe", "ob3"],
            vec!["prog", "test1", "-o", "ob1", "ob2"],
            vec!["prog", "test1", "--cc", "abc"],
            vec!["prog", "test1", "-soa", "soa"],
            vec!["prog", "test3"],
            vec!["prog", "test3", "--oe", "oe"],
            vec!["prog", "test3", "--of", "of"],
            vec!["prog", "test3", "--cb", "y"],
        ]
    );
}

#[test]
fn option_shorts() {
    snapshot_multi!(
        SCRIPT_OPTIONS,
        [
            vec!["prog", "test1", "-af"],
            vec!["prog", "test1", "-ae", "e"],
            vec!["prog", "test1", "-afe", "e"],
            vec!["prog", "test1", "-ao", "v1", "v2"],
        ]
    );
}

#[test]
fn arg_eval() {
    snapshot_multi!(
        SCRIPT_ARGS,
        [
            vec!["prog", "cmd_arg", "v1"],
            vec!["prog", "cmd_multi_arg", "v1", "v2"],
            vec!["prog", "cmd_arg_with_default"],
            vec!["prog", "cmd_arg_with_default", "v1"],
            vec!["prog", "cmd_arg_with_default_fn"],
            vec!["prog", "cmd_arg_with_choices", "x"],
            vec!["prog", "cmd_arg_with_choice_fn", "abc"],
            vec!["prog", "cmd_two_multi_args", "v1", "v2"],
            vec!["prog", "cmd_two_multi_args", "v1", "v2", "v3"],
            vec!["prog", "cmd_three_required_args", "v1", "v2", "v3"],
        ]
    );
}

#[test]
fn arg_subcmd_help() {
    let names: Vec<String> = SCRIPT_ARGS
        .lines()
        .filter(|v| v.starts_with("cmd") && v.contains("() {"))
        .map(|v| v.split_once('(').unwrap().0.to_string())
        .collect();

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
            vec!["prog", "cmd", "v1", "v2"],
            vec!["prog", "cmd", "--o1", "-o2", "-3"]
        ]
    );
}

#[test]
fn arg_no_option() {
    snapshot!(SCRIPT_ARGS, &["prog", "cmd_multi_arg", "--o1", "-o2", "-3"]);
}

#[test]
fn arg_two_multi() {
    snapshot_multi!(
        SCRIPT_ARGS,
        [
            vec!["prog", "cmd_two_multi_args", "a", "b", "c"],
            vec!["prog", "cmd_two_multi_args", "--", "a", "b", "c"],
            vec!["prog", "cmd_two_multi_args", "a", "--", "b", "c"],
            vec!["prog", "cmd_two_multi_args", "a", "b", "--", "c"],
            vec!["prog", "cmd_two_multi_args", "a", "b", "c", "--"],
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
cmda() { :; }
"###;
    snapshot!(script, &["prog", "cmda", "-h"]);
}

#[test]
fn option_prefixed() {
    let script = r###"
# @option -o-
# @option -D-*
"###;
    snapshot!(script, &["prog", "-o1", "-Dv1=foo", "-Dv2", "bar"]);
}

#[test]
fn option_assigned() {
    let script = r###"
# @option --oa:
# @option --ob:*
# @option --oc: <VALUE?>
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "--oa=v1", "--ob=1", "--ob=2"],
            vec!["prog", "--oa", "v1"],
            vec!["prog", "--oc", "v1"],
        ]
    );
}

#[test]
fn cmd_with_hyphen() {
    let script = r###"
# @cmd Run --foo
# @flag --fa
--foo() { :; }

# @cmd Run bar
# @alias -B
# @flag --fa
bar() { :; }
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
-A() { :; }
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
cmda() { :; }

# @cmd
# @option --ob[x|y]  desc 2
cmdb() { :; }
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
fn notation_modifier() {
    let script = r###"
# @option --oa <VALUE*>           multi values, zero or more
# @option --ob <VALUE+>           multi values, one or more
# @option --oc <VALUE?>           zero or one
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "--oa"],
            vec!["prog", "--oa", "v1"],
            vec!["prog", "--oa", "v1", "v2"],
            vec!["prog", "--ob"],
            vec!["prog", "--ob", "v1"],
            vec!["prog", "--ob", "v1", "v2"],
            vec!["prog", "--oc"],
            vec!["prog", "--oc", "v1"],
            vec!["prog", "--oc", "v1", "v2"],
        ]
    );
}

#[test]
fn default_subcommand() {
    let script = r###"
# @cmd
# @meta default-subcommand
cmda() { :; }

# @cmd
cmdb() { :; }
"###;
    snapshot_multi!(
        script,
        [
            vec!["prog", "-h"],
            vec!["prog", "v1"],
            vec!["prog", "cmda", "v1"],
            vec!["prog", "cmdb", "v1"],
        ]
    );
}
