#[test]
fn test_complete() {
    let script = r###"
# @flag --foo


# @cmd
cmd() {
}

    "###;
    complete!(script, "prog");
}