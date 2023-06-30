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
        let err = argc::eval($source, &args, None, None).unwrap_err();
        assert_eq!(err.to_string().as_str(), $err);
    };
}

#[macro_export]
macro_rules! snapshot {
    ($source:expr, $args:expr) => {
        let (script_path, script_content, script_file) =
            $crate::fixtures::create_argc_script($source, "script.sh");
        snapshot!(&script_content, $args, Some(script_path.as_str()), None);
        script_file.close().unwrap();
    };
    (
        $source:expr,
        $args:expr,
		$path:expr,
		$width:expr
    ) => {
        let args: Vec<String> = $args.iter().map(|v| v.to_string()).collect();
        let values = argc::eval($source, &args, $path, $width).unwrap();
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
        let (script_path, script_content, script_file) =
            $crate::fixtures::create_argc_script($source, "script.sh");
        for args in $matrix.iter() {
            let args: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            let values =
                argc::eval(&script_content, &args, Some(script_path.as_str()), None).unwrap();
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
        script_file.close().unwrap();
        insta::assert_snapshot!(data);
    };
}

#[macro_export]
macro_rules! snapshot_compgen {
    (
		$source:expr,
        $matrix:expr,
        $shell:expr
    ) => {
        let mut data = String::new();
        let (script_path, script_content, script_file) =
            $crate::fixtures::create_argc_script($source, "compgen.sh");
        for args in $matrix.iter() {
            let args: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            let words = match argc::compgen($shell, &script_path, &script_content, &args, false) {
                Ok(stdout) => stdout,
                Err(stderr) => stderr.to_string(),
            };
            let piece = format!(
                r###"************ COMPGEN `{}` ************
{}

"###,
                args.join(" "),
                words
            );
            data.push_str(&piece);
        }
        script_file.close().unwrap();
        insta::assert_snapshot!(data);
    };
    (
		$source:expr,
        $matrix:expr
    ) => {
        snapshot_compgen!($source, $matrix, argc::Shell::Generic);
    };
}

#[macro_export]
macro_rules! snapshot_compgen_shells {
    (
		$source:expr,
        $args:expr
    ) => {
        let mut data = String::new();
        let (script_path, script_content, script_file) =
            $crate::fixtures::create_argc_script($source, "compgen.sh");
        let args: Vec<String> = $args.iter().map(|v| v.to_string()).collect();
        for shell in argc::Shell::list() {
            let words = match argc::compgen(shell, &script_path, &script_content, &args, false) {
                Ok(stdout) => stdout,
                Err(stderr) => stderr.to_string(),
            };
            let piece = format!(
                r###"************ COMPGEN {:?} `{}` ************
{}

"###,
                shell,
                args.join(" "),
                words
            );
            data.push_str(&piece);
        }
        script_file.close().unwrap();
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
