#[test]
fn test_unsupported_tag() {
    let script = r###"
# @baz
    "###;
    fatal!(script, &["prog"], "@baz(line 2) is unknown");
}

#[test]
fn test_unexpected_arg() {
    let script = r###"
# @flag --foo
foo() {

}

# @arg baz
    "###;
    fatal!(
        script,
        &["prog"],
        "@arg(line 7) is unexpected, maybe miss @cmd?"
    );
}

#[test]
fn test_conflict_cmd() {
    let script = r###"
# @flag --foo

# @cmd
foo() {
}

# @cmd
foo() {
}
    "###;
    fatal!(
        script,
        &["prog"],
        "foo(line 9) is conflicted with cmd or alias at line 5"
    );
}

#[test]
fn test_conflict_short_option() {
    let script = r###"
# @option -f --foo1
# @option -f --foo2
    "###;
    fatal!(
        script,
        &["prog"],
        "@option(line 3) has '-f' already exists at line 2"
    );
}

#[test]
fn test_conflict_long_option() {
    let script = r###"
# @option -a --foo
# @option -f --foo
    "###;
    fatal!(
        script,
        &["prog"],
        "@option(line 3) has '--foo' already exists at line 2"
    );
}

#[test]
fn test_conflict_short_flag() {
    let script = r###"
# @flag -f --foo1
# @flag -f --foo2
    "###;
    fatal!(
        script,
        &["prog"],
        "@flag(line 3) has '-f' already exists at line 2"
    );
}

#[test]
fn test_conflict_long_flag() {
    let script = r###"
# @flag -a --foo
# @flag -f --foo
    "###;
    fatal!(
        script,
        &["prog"],
        "@flag(line 3) has '--foo' already exists at line 2"
    );
}

#[test]
fn test_conflict_positional() {
    let script = r###"
# @arg foo
# @arg foo
    "###;
    fatal!(
        script,
        &["prog"],
        "@arg(line 3) has 'foo' already exists at line 2"
    );
}

#[test]
fn test_conflict_alias() {
    let script = r###"
# @cmd
# @alias t,tst
test() {
}
# @cmd
# @alias t
try() {
}
    "###;
    fatal!(
        script,
        &["prog"],
        "@alias(line 7) is conflicted with cmd or alias at line 3"
    );
}

#[test]
fn test_option_miss_default_fn() {
    let script = r###"
# @option --foo=`_fn`
    "###;
    fatal!(script, &["prog"], "_fn(line 2) is missing");
}

#[test]
fn test_option_miss_choice_fn() {
    let script = r###"
# @option --foo[`_fn`]
    "###;
    fatal!(script, &["prog"], "_fn(line 2) is missing");
}

#[test]
fn test_arg_miss_default_fn() {
    let script = r###"
# @arg foo=`_fn`
    "###;
    fatal!(script, &["prog"], "_fn(line 2) is missing");
}

#[test]
fn test_arg_miss_choice_fn() {
    let script = r###"
# @arg foo[`_fn`]
    "###;
    fatal!(script, &["prog"], "_fn(line 2) is missing");
}

#[test]
fn test_cmd_miss_fn() {
    let script = r###"
# @cmd
# @cmd
    "###;
    fatal!(script, &["prog"], "@cmd(line 2) miss function?");
}
