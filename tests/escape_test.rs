const SCRIPT: &str = r###"
# @arg value! a test value
"###;

#[test]
fn test_syntax_error() {
    plain!(SCRIPT, &["prog", "$foo"], stdout: "argc_value=\\$foo",);
    plain!(SCRIPT, &["prog", "`pwd`"], stdout: "argc_value=\\`pwd\\`",);
    plain!(SCRIPT, &["prog", "$(pwd)"], stdout: "argc_value=\\$\\(pwd\\)",);
    plain!(SCRIPT, &["prog", "'"], stdout: "argc_value=\\'",);
    plain!(SCRIPT, &["prog", "\\1"], stdout: "argc_value=\\\\1",);
    plain!(SCRIPT, &["prog", ""], stdout: "argc_value=''",);
    plain!(SCRIPT, &["prog", "\n"], stdout: "argc_value='\n'",);
}
