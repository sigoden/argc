#[test]
fn test_no_main_no_subcmds() {
    let script = r###"
# @flag --foo
    "###;
    plain!(script, &["prog"], stdout: "",);
}

#[test]
fn test_with_main_no_subcmds() {
    let script = r###"
# @flag --foo

main() {

}
    "###;
    plain!(script, &["prog"], stdout: "",);
}

#[test]
fn test_with_main_and_subcmds() {
    let script = r###"
# @flag --foo


# @cmd
cmd() {
}

main() {

}
    "###;
    plain!(script, &["prog"], stdout: "",);
    plain!(script, &["prog", "cmd"], stdout: "",);
    snapshot!(script, &["prog", "-h"],);
}

#[test]
fn test_without_main_but_with_subcmds() {
    let script = r###"
# @flag --foo


# @cmd
cmd() {
}

    "###;
    plain!(script, &["prog", "cmd"], stdout: "",);
    snapshot!(script, &["prog"],);
}

#[test]
fn test_without_main_but_with_subcmds2() {
    let script = r###"
# @flag --foo


# @cmd
cmd() {
}

    "###;
    snapshot!(script, &["prog", "-h"],);
}
