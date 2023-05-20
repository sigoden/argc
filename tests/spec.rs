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
        vec![
            vec!["prog", "cmda"],
            vec!["prog", "cmda", "-a"],
            vec!["prog", "cmda", "-f", "-f"],
            vec!["prog", "cmda", "-e", "e"],
            vec!["prog", "cmda", "--oa", "oa"],
            vec!["prog", "cmda", "--ob", "ob1", "--ob", "ob2"],
            vec!["prog", "cmda", "--ob", "ob1", "ob2"],
            vec!["prog", "cmda", "-o", "ob1", "ob2"],
            vec!["prog", "cmda", "--cc", "abc"],
            vec!["prog", "cmda", "-soa", "soa"],
            vec!["prog", "cmda", "--ob", "a", "b", "--ob", "c"],
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
        vec![
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
        vec![
            vec!["prog", "cmdb", "v1"],
            vec!["prog", "cmdc", "v1", "v2"],
            vec!["prog", "cmdf"],
            vec!["prog", "cmdf", "v1"],
            vec!["prog", "cmdg"],
            vec!["prog", "cmdh", "x"],
            vec!["prog", "cmdj", "xyz"],
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
        vec![
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
        vec![
            vec!["prog", "cmdp", "a", "b", "c"],
            vec!["prog", "cmdp", "--", "a", "b", "c"],
            vec!["prog", "cmdp", "a", "--", "b", "c"],
            vec!["prog", "cmdp", "a", "b", "--", "c"],
            vec!["prog", "cmdp", "a", "b", "c", "--"],
        ]
    );
}

#[test]
fn dashdash_split() {
    let script = r###"
# @flag -f
# @option --oa
# @arg v1
# @arg v2*
"###;
    snapshot_multi!(
        script,
        vec![
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
