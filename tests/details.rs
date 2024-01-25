use super::SCRIPT_DETAILS;

#[test]
fn wrap() {
    snapshot!(SCRIPT_DETAILS, &["prog", "-h"], None, Some(80));
}

#[test]
fn wrap2() {
    snapshot!(SCRIPT_DETAILS, &["prog", "foo", "-h"], None, Some(80));
}

#[test]
fn nowrap() {
    snapshot!(SCRIPT_DETAILS, &["prog", "-h"], None, None);
}
