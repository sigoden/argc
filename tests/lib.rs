#[macro_export]
macro_rules! argc {
    (
        source: $source:expr,
        args: $args:expr,
        $(stdout: $stdout:expr,)?
        $(stderr: $stderr:expr,)?
    ) => {
        let cli = argc::Cli::from_str($source).unwrap();
        let output = cli.eval($args);
        $(assert_eq!(output.1, $stderr);)?
        $(assert_eq!(output.0, $stdout);)?

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
