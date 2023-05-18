#[macro_export]
macro_rules! snapshot {
    (
		$path:expr,
        $source:expr,
        $args:expr
    ) => {
        let args: Vec<String> = $args.iter().map(|v| v.to_string()).collect();
        let values = argc::eval($path, $source, &args, None).unwrap();
        let output = argc::ArgcValue::to_shell(values);
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
macro_rules! snapshot_spec {
    ($args:expr) => {
        let (path, source) = $crate::fixtures::get_spec();
        snapshot!(Some(path.as_str()), source.as_str(), $args);
    };
}

#[macro_export]
macro_rules! plain {
    (
        $source:expr,
        $args:expr,
		$output:expr
    ) => {
        let args: Vec<String> = $args.iter().map(|v| v.to_string()).collect();
        let values = argc::eval(None, $source, &args, None).unwrap();
        let output = argc::ArgcValue::to_shell(values);
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
        let args: Vec<String> = $args.iter().map(|v| v.to_string()).collect();
        let err = argc::eval(None, $source, &args, None).unwrap_err();
        assert_eq!(err.to_string().as_str(), $err);
    };
}

#[macro_export]
macro_rules! snapshot_compgen {
    (
        $line:expr
    ) => {
        let (script_file, script_content) = $crate::fixtures::get_spec();
        let (stdout, stderr) = match argc::compgen(
            argc::Shell::Fish,
            &script_file,
            &script_content,
            "test",
            $line,
        ) {
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
