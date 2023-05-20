#[macro_export]
macro_rules! eval {
    (
        $source:expr,
        $args:expr,
		$output:expr
    ) => {
        let args: Vec<String> = $args.iter().map(|v| v.to_string()).collect();
        let values = argc::eval(None, $source, &args, None).unwrap();
        let shell_code = argc::ArgcValue::to_shell(values);
        assert_eq!(shell_code, $output);
    };
}

#[macro_export]
macro_rules! fail {
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
macro_rules! snapshot {
    ($source:expr, $args:expr) => {
        snapshot!(None, $source, $args, None);
    };
    (CREATE, $source:expr, $args:expr) => {
        let tmpdir = assert_fs::TempDir::new().unwrap();
        let script_content = if !$source.contains("--argc-eval") {
            format!(
                r###"{}
eval "$(argc --argc-eval "$0" "$@")"
"###,
                $source
            )
        } else {
            $source.to_string()
        };
        let child = assert_fs::fixture::PathChild::child(&tmpdir, "script.sh");
        assert_fs::fixture::FileWriteStr::write_str(&child, &script_content).unwrap();
        let script_file = child.path().to_string_lossy().to_string();
        snapshot!(Some(script_file.as_str()), &script_content, $args, None);
    };
    ($path:expr, $source:expr, $args:expr) => {
        snapshot!(Some($path), $source, $args, None);
    };
    (
		$path:expr,
        $source:expr,
        $args:expr,
		$width:expr
    ) => {
        let args: Vec<String> = $args.iter().map(|v| v.to_string()).collect();
        let values = argc::eval($path, $source, &args, $width).unwrap();
        let shell_code = argc::ArgcValue::to_shell(values);
        let args = $args.join(" ");
        let data = format!(
            r###"RUN
{}

OUTPUT
{}
"###,
            args, shell_code,
        );
        insta::assert_snapshot!(data);
    };
}

#[macro_export]
macro_rules! snapshot_multi {
    (
		$source:expr,
		$matrix:expr
	) => {
        let mut data = String::new();
        for args in $matrix.iter() {
            let args: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            let values = argc::eval(None, $source, &args, None).unwrap();
            let args = args.join(" ");
            let piece = format!(
                r###"************ RUN ************
{}

OUTPUT
{}

"###,
                args,
                argc::ArgcValue::to_shell(values),
            );
            data.push_str(&piece);
        }
        insta::assert_snapshot!(data);
    };
}

#[macro_export]
macro_rules! snapshot_compgen {
    (
		$source:expr,
        $matrix:expr
    ) => {
        let mut data = String::new();
        let tmpdir = assert_fs::TempDir::new().unwrap();
        let script_content = if !$source.contains("--argc-eval") {
            format!(
                r###"{}
eval "$(argc --argc-eval "$0" "$@")"
"###,
                $source
            )
        } else {
            $source.to_string()
        };
        let script_file = {
            let child = assert_fs::fixture::PathChild::child(&tmpdir, "compgen.sh");
            assert_fs::fixture::FileWriteStr::write_str(&child, &script_content).unwrap();
            child.path().to_string_lossy().to_string()
        };
        for line in $matrix.iter() {
            let words = match argc::compgen(
                argc::Shell::Fish,
                &script_file,
                &script_content,
                "test",
                line,
            ) {
                Ok(stdout) => stdout,
                Err(stderr) => stderr.to_string(),
            };
            let piece = format!(
                r###"************ COMPGEN `prog {}` ************
{}

"###,
                line, words
            );
            data.push_str(&piece);
        }
        insta::assert_snapshot!(data);
    };
}

#[macro_export]
macro_rules! snapshot_export {
    ($source:expr) => {
        let json = argc::export($source).unwrap();
        let output = serde_json::to_string_pretty(&json).unwrap();
        insta::assert_snapshot!(output);
    };
}
