#[test]
fn test_unsupport_tag() {
    let script = r###"
# @baz
    "###;
    fatal!(script, &["prog"], "@baz(line 2) is invalid");
}

#[test]
fn test_invalid_tag() {
    let script = r###"
# @flag -f
    "###;
    fatal!(script, &["prog"], "@flag(line 2) is invalid");
}


#[test]
fn test_unexpect_arg() {
    let script = r###"
# @flag --foo
foo() {

}

# @arg baz
    "###;
    fatal!(script, &["prog"], "@arg(line 7) is unexpected, maybe miss @cmd?");
}


#[test]
fn test_redefined_fn() {
//     let script = r###"
// # @flag --foo

// @cmd
// foo() {
// }

// @cmd
// foo() {

// }
//     "###;
//     fatal!(script, &["prog"], "");
}


#[test]
fn test_conflict_flag() {
}
#[test]
fn test_conflict_option() {
}
#[test]
fn test_conflict_positional() {
}