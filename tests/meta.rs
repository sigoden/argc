#[test]
fn dotenv() {
    let script = r###"
# @meta dotenv
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn dotenv_custom_path() {
    let script = r###"
# @meta dotenv .env.local
"###;
    snapshot!(script, &["prog"]);
}

#[test]
fn binname() {
    let script = r###"
# @meta binname test-binname
"###;
    snapshot!(script, &["prog", "-h"]);
}
