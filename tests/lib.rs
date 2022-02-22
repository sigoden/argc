use insta::assert_snapshot;

macro_rules! assert_argc {
    (
        $source:expr,
        $args:expr
    ) => {
        let (stdout, stderr) = argc::run($source, $args).unwrap();
        let args = $args.join(" ");
        let stdout = stdout.unwrap_or_default();
        let stderr = stderr.unwrap_or_default();
        let output = format!(r###"RUN
{}

STDOUT
{}

STDERR
{}
"###, args, stdout, stderr);
        assert_snapshot!(output);
    };
}

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