use crate::*;

#[test]
fn escape() {
    snapshot!(
        SCRIPT_ARGS,
        &["prog", "cmd", "$foo", "`pwd`", "$(pwd)", "'", "\\1", "", "\n", "世界", " "]
    );
}
