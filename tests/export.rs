use crate::*;

#[test]
fn case1() {
    let script = r###"
# @describe Test argc
# @version    1.0.0
# @author     nobody <nobody@example.com>
# @cmd
# @alias a
# @flag   -b --fa A flag
# @option -o --oa A option
# @arg var
cmd() { :? }
"###;
    snapshot_export!(script);
}
