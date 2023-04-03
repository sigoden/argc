const SCRIPT: &str = r###"
# @arg value! a test value
"###;

#[test]
fn test_syntax_error() {
    plain!(SCRIPT, &["prog", "$foo"], "argc_value='$foo'");
    plain!(SCRIPT, &["prog", "`pwd`"], "argc_value='`pwd`'");
    plain!(SCRIPT, &["prog", "$(pwd)"], "argc_value='$(pwd)'");
    plain!(SCRIPT, &["prog", "'"], "argc_value=''\\'''");
    plain!(SCRIPT, &["prog", "\\1"], "argc_value='\\1'");
    plain!(SCRIPT, &["prog", ""], "argc_value=''");
    plain!(SCRIPT, &["prog", "\n"], "argc_value='\n'");
    plain!(SCRIPT, &["prog", "世界"], "argc_value=世界");
    plain!(SCRIPT, &["prog", " "], "argc_value=' '");
}
