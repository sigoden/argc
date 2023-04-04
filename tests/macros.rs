#[macro_export]
macro_rules! snapshot {
    (
        $source:expr,
        $args:expr
    ) => {
        let values = argc::eval($source, $args).unwrap();
        let output = argc::ArgcValue::to_shell(values, true);
        let args = $args.join(" ");
        let output = format!(
            r###"RUN
{}

OUTPUT
{}
"###,
            args, output,
        );
        insta::assert_snapshot!(output);
    };
}

#[macro_export]
macro_rules! plain {
    (
        $source:expr,
        $args:expr,
		$output:expr
    ) => {
        let values = argc::eval($source, $args).unwrap();
        let output = argc::ArgcValue::to_shell(values, true);
        assert_eq!(output, $output);
    };
}

#[macro_export]
macro_rules! fatal {
    (
        $source:expr,
        $args:expr,
        $err:expr
    ) => {
        let err = argc::eval($source, $args).unwrap_err();
        assert_eq!(err.to_string().as_str(), $err);
    };
}

#[macro_export]
macro_rules! snapshot_compgen {
    (
        $line:expr
    ) => {
        let (script_file, script_content) = $crate::fixtures::get_spec();
        let (stdout, stderr) =
            match argc::compgen(argc::Shell::Fish, &script_file, &script_content, $line) {
                Ok(stdout) => (stdout, String::new()),
                Err(stderr) => (String::new(), stderr.to_string()),
            };

        let output = format!(
            r###"RUN
{}

STDOUT
{}

STDERR
{}
"###,
            $line, stdout, stderr
        );
        insta::assert_snapshot!(output);
    };
}

#[macro_export]
macro_rules! snapshot_export {
    ($source:expr, $name:literal) => {
        let json = argc::export($source, $name).unwrap();
        let output = serde_json::to_string_pretty(&json).unwrap();
        insta::assert_snapshot!(output);
    };
}
