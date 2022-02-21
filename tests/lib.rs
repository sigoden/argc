#[macro_export]
macro_rules! argc {
    (
        source: $source:expr,
        args: $args:expr,
        $(stdout: $stdout:expr,)?
        $(stderr: $stderr:expr,)?
    ) => {
        let output = argc::eval($source, $args).unwrap();
        $(assert_eq!(output.1.unwrap(), $stderr);)?
        $(assert_eq!(output.0.unwrap(), $stdout);)?

    };
}

#[test]
fn test_git_help() {
    argc!(
       source: include_str!("git.sh"),
       args: &["git", "-h"],
       stdout: "exit 1",
       stderr: include_str!("git.help.txt"),
    );
}

#[test]
fn test_git_add_help() {
    argc!(
       source: include_str!("git.sh"),
       args: &["git", "add", "-h"],
       stderr: include_str!("git.add.help.txt"),
    );
}

#[test]
fn test_git_remote_help() {
    argc!(
       source: include_str!("git.sh"),
       args: &["git", "push", "-h"],
       stderr: include_str!("git.push.help.txt"),
    );
}

#[test]
fn test_git_log_help() {
    argc!(
       source: include_str!("git.sh"),
       args: &["git", "log", "-h"],
       stderr: include_str!("git.log.help.txt"),
    );
}
