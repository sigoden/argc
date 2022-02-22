use insta::assert_snapshot;

#[test]
fn test_git() {
    assert_argc!(include_str!("git.sh"), &["git", "-h"]);
}

#[test]
fn test_git_add() {
    assert_argc!(include_str!("git.sh"), &["git", "add", "-h"]);
}

#[test]
fn test_git_remote() {
    assert_argc!(include_str!("git.sh"), &["git", "push", "-h"]);
}

#[test]
fn test_git_log() {
    assert_argc!(include_str!("git.sh"), &["git", "log", "-h"]);
}
