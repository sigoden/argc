#[macro_export]
macro_rules! test_help {
    (
        source: $s:expr,
        args: $a:expr,
        help: $h:expr,
    ) => {
        let cli = argc::Cli::from_str($s).unwrap();
        let app = cli.build($a[0]);
        let res = app.try_get_matches_from($a);
        let err = res.unwrap_err();
        assert_eq!(err.to_string(), $h)
    };
}

#[test]
fn test_git_help() {
    test_help!(
       source: include_str!("git.sh"),
       args: ["git", "-h"],
       help: include_str!("git.help.txt"),
    );
}

#[test]
fn test_git_add_help() {
    test_help!(
       source: include_str!("git.sh"),
       args: ["git", "add", "-h"],
       help: include_str!("git.add.help.txt"),
    );
}

#[test]
fn test_git_remote_help() {
    test_help!(
       source: include_str!("git.sh"),
       args: ["git", "push", "-h"],
       help: include_str!("git.push.help.txt"),
    );
}

#[test]
fn test_git_log_help() {
    test_help!(
       source: include_str!("git.sh"),
       args: ["git", "log", "-h"],
       help: include_str!("git.log.help.txt"),
    );
}
