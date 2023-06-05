use crate::*;

#[test]
fn case1() {
    snapshot_multi!(
        SCRIPT_OPTIONS,
        vec![
            vec!["prog", "_choice_fn"],
            vec!["prog", "_choice_fn", "prog cmda --cc "],
            vec!["prog", "_choice_fn", "prog cmda -a --oa oa --cc "],
        ]
    );
}

#[test]
fn case2() {
    snapshot_multi!(
        SCRIPT_ARGS,
        vec![
            vec!["prog", "_choice_fn"],
            vec!["prog", "_choice_fn", "prog cmdl "],
            vec!["prog", "_choice_fn", "prog cmdl v1"],
            vec!["prog", "_choice_fn", "prog cmdl v1 "],
            vec!["prog", "_choice_fn", "prog cmdl v1 v2"],
            vec!["prog", "_choice_fn", "prog cmdl v1 v2 "],
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
            vec!["prog", "_choice_fn", "prog  "],
            vec!["prog", "_choice_fn", "prog v1"],
            vec!["prog", "_choice_fn", "prog v1 "],
            vec!["prog", "_choice_fn", "prog v1 v2"],
            vec!["prog", "_choice_fn", "prog v1 v2 "],
        ]
    );
}
