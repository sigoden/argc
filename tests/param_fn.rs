use crate::*;

#[test]
fn case1() {
    snapshot_multi!(
        SCRIPT_OPTIONS,
        vec![
            vec!["prog", "_choice_fn"],
            vec!["prog", "_choice_fn", "cmda --cc "],
            vec!["prog", "_choice_fn", "cmda -a --oa oa --cc "],
        ]
    );
}

#[test]
fn case2() {
    snapshot_multi!(
        SCRIPT_ARGS,
        vec![
            vec!["prog", "_choice_fn"],
            vec!["prog", "_choice_fn", "cmdl "],
            vec!["prog", "_choice_fn", "cmdl v1"],
            vec!["prog", "_choice_fn", "cmdl v1 "],
            vec!["prog", "_choice_fn", "cmdl v1 v2"],
            vec!["prog", "_choice_fn", "cmdl v1 v2 "],
        ]
    );
}

#[test]
fn case3() {
    let script = r###"
# @arg v1![`_choice_fn`]
# @arg v2![`_choice_fn`]
_choice_fn() {
	echo a
	echo b
}
"###;
    snapshot_multi!(
        script,
        vec![
            vec!["prog", "_choice_fn"],
            vec!["prog", "_choice_fn", " "],
            vec!["prog", "_choice_fn", "v1"],
            vec!["prog", "_choice_fn", "v1 "],
            vec!["prog", "_choice_fn", "v1 v2"],
            vec!["prog", "_choice_fn", "v1 v2 "],
        ]
    );
}
