#[macro_export]
macro_rules! assert_argc {
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
        assert_snapshot!(output);
    };
}
