use crate::*;

#[test]
fn case1() {
    snapshot_multi!(
        SCRIPT_OPTIONS,
        [
            vec!["prog", "___internal___", "_choice_fn"],
            vec![
                "prog",
                "___internal___",
                "_choice_fn",
                "prog",
                "test1",
                "--cc",
                ""
            ],
            vec![
                "prog",
                "___internal___",
                "_choice_fn",
                "prog",
                "test1",
                "-a",
                "--oa",
                "oa",
                "--cc",
                ""
            ],
        ]
    );
}

#[test]
fn case2() {
    snapshot_multi!(
        SCRIPT_ARGS,
        [
            vec!["prog", "___internal___", "_choice_fn"],
            vec![
                "prog",
                "___internal___",
                "_choice_fn",
                "prog",
                "cmd_multi_arg_with_choice_fn",
                ""
            ],
            vec![
                "prog",
                "___internal___",
                "_choice_fn",
                "prog",
                "cmd_multi_arg_with_choice_fn",
                "v1"
            ],
            vec![
                "prog",
                "___internal___",
                "_choice_fn",
                "prog",
                "cmd_multi_arg_with_choice_fn",
                "v1",
                ""
            ],
            vec![
                "prog",
                "___internal___",
                "_choice_fn",
                "prog",
                "cmd_multi_arg_with_choice_fn",
                "v1",
                "v2"
            ],
            vec![
                "prog",
                "___internal___",
                "_choice_fn",
                "prog",
                "cmd_multi_arg_with_choice_fn",
                "v1",
                "v2",
                ""
            ],
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
        [
            vec!["prog", "___internal___", "_choice_fn"],
            vec!["prog", "___internal___", "_choice_fn", "prog", ""],
            vec!["prog", "___internal___", "_choice_fn", "prog", "v1"],
            vec!["prog", "___internal___", "_choice_fn", "prog", "v1", ""],
            vec!["prog", "___internal___", "_choice_fn", "prog", "v1", "v2"],
            vec![
                "prog",
                "___internal___",
                "_choice_fn",
                "prog",
                "v1",
                "v2",
                ""
            ],
        ]
    );
}
