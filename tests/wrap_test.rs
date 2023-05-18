macro_rules! snapshort_wrap {
    ($source:expr, $width:expr) => {
        let args: Vec<String> = vec!["--help".into()];
        let values = argc::eval(None, $source, &args, Some($width)).unwrap();
        let output = argc::ArgcValue::to_shell(values);
        insta::assert_snapshot!(output);
    };
}

#[test]
fn test_wrap() {
    let script: &str = r###"
# @option --foo Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Neque laoreet suspendisse libero id. 
# @arg target Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Sed viverra tellus in hac habitasse platea.
# @cmd Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Arcu cursus euismod quis viverra. 
foo() {
}
"###;
    snapshort_wrap!(script, 80);
}
