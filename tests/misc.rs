use crate::*;

#[test]
fn escape() {
    snapshot!(
        SCRIPT_ARGS,
        &["prog", "cmda", "$foo", "`pwd`", "$(pwd)", "'", "\\1", "", "\n", "世界", " "]
    );
}
