#[macro_export]
macro_rules! snapshot {
    (
        $source:expr,
        $args:expr
    ) => {
        let (stdout, stderr) = argc::run($source, $args).unwrap();
        let args = $args.join(" ");
        let stdout = stdout.unwrap_or_default();
        let stderr = stderr.unwrap_or_default();
        let output = format!(
            r###"RUN
{}

STDOUT
{}

STDERR
{}
"###,
            args, stdout, stderr
        );
        insta::assert_snapshot!(output);
    };
}

#[macro_export]
macro_rules! plain {
    (
        $source:expr,
        $args:expr,
        $(stdout: $stdout:expr,)?
        $(stderr: $stderr:expr,)?
    ) => {
        let result = argc::run($source, $args).unwrap();
        $({
            assert_eq!(result.0.unwrap_or_default().as_str(), $stdout);
        })?
        $({
            assert_eq!(result.1.unwrap_or_default().as_str(), $stderr);
        })?
    };
}


#[macro_export]
macro_rules! fatal {
    (
        $source:expr,
        $args:expr,
        $err:expr
    ) => {
        let err = argc::run($source, $args).unwrap_err();
        assert_eq!(err.to_string().as_str(), $err);
    }
}