use super::SCRIPT_MULTILINE;

#[test]
fn wrap() {
    snapshot!(SCRIPT_MULTILINE, &["prog", "-h"], None, Some(80));
}

#[test]
fn wrap2() {
    snapshot!(SCRIPT_MULTILINE, &["prog", "foo", "-h"], None, Some(80));
}

#[test]
fn nowrap() {
    snapshot!(SCRIPT_MULTILINE, &["prog", "-h"], None, None);
}
