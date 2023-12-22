use crate::utils::escape_shell_words;

pub const VARIABLE_PREFIX: &str = "argc";
pub const INIT_HOOK: &str = "_argc_init";

#[derive(Debug, PartialEq, Eq)]
pub enum ArgcValue {
    Single(String, String),
    SingleFn(String, String),
    Multiple(String, Vec<String>),
    PositionalSingle(String, String),
    PositionalSingleFn(String, String),
    PositionalMultiple(String, Vec<String>),
    ExtraPositionalMultiple(Vec<String>),
    InitHook,
    Dotenv(String),
    CmdFn(String),
    ParamFn(String),
    Error((String, i32)),
}

impl ArgcValue {
    pub fn to_shell(values: &[Self]) -> String {
        let mut output = vec![];
        let mut last = String::new();
        let mut positional_args = vec![];
        let mut init_hook = false;
        for value in values {
            match value {
                ArgcValue::Single(name, value) => {
                    output.push(format!(
                        "{}_{}={}",
                        VARIABLE_PREFIX,
                        sanitize_arg_name(name),
                        escape_shell_words(value)
                    ));
                }
                ArgcValue::SingleFn(name, fn_name) => {
                    output.push(format!(
                        "{}_{}=`{}`",
                        VARIABLE_PREFIX,
                        sanitize_arg_name(name),
                        fn_name,
                    ));
                }
                ArgcValue::Multiple(name, values) => {
                    output.push(format!(
                        "{}_{}=( {} )",
                        VARIABLE_PREFIX,
                        sanitize_arg_name(name),
                        values
                            .iter()
                            .map(|v| escape_shell_words(v))
                            .collect::<Vec<String>>()
                            .join(" ")
                    ));
                }
                ArgcValue::PositionalSingle(name, value) => {
                    let value = escape_shell_words(value);
                    output.push(format!(
                        "{}_{}={}",
                        VARIABLE_PREFIX,
                        sanitize_arg_name(name),
                        &value
                    ));
                    positional_args.push(value);
                }
                ArgcValue::PositionalSingleFn(name, fn_name) => {
                    output.push(format!(
                        "{}_{}=`{}`",
                        VARIABLE_PREFIX,
                        sanitize_arg_name(name),
                        &fn_name
                    ));
                    positional_args.push(format!("`{}`", fn_name));
                }
                ArgcValue::PositionalMultiple(name, values) => {
                    let values = values
                        .iter()
                        .map(|v| escape_shell_words(v))
                        .collect::<Vec<String>>();
                    output.push(format!(
                        "{}_{}=( {} )",
                        VARIABLE_PREFIX,
                        sanitize_arg_name(name),
                        values.join(" ")
                    ));
                    positional_args.extend(values);
                }
                ArgcValue::ExtraPositionalMultiple(values) => {
                    let values = values
                        .iter()
                        .map(|v| escape_shell_words(v))
                        .collect::<Vec<String>>();
                    positional_args.extend(values);
                }
                ArgcValue::InitHook => {
                    init_hook = true;
                }
                ArgcValue::Dotenv(value) => {
                    let value = if value.is_empty() {
                        ".env".to_string()
                    } else {
                        escape_shell_words(value)
                    };
                    output.push(format!(
                        "[ -f {value} ] && set -o allexport && source {value} && set +o allexport"
                    ))
                }
                ArgcValue::CmdFn(name) => {
                    if positional_args.is_empty() {
                        last = name.to_string();
                    } else {
                        last = format!("{} {}", name, positional_args.join(" "));
                    }
                    output.push(format!("{}__fn={}", VARIABLE_PREFIX, name));
                }
                ArgcValue::ParamFn(name) => {
                    if positional_args.is_empty() {
                        last = format!("{name};exit;");
                    } else {
                        last = format!("{} {};exit;", name, positional_args.join(" "));
                    }
                }
                ArgcValue::Error((error, exit)) => {
                    return format!("command cat >&2 <<-'EOF' \n{}\nEOF\nexit {}", error, exit)
                }
            }
        }

        output.push(format!(
            "{}__positionals=( {} )",
            VARIABLE_PREFIX,
            positional_args.join(" ")
        ));
        if init_hook {
            output.push(INIT_HOOK.to_string())
        }
        if !last.is_empty() {
            output.push(last);
        }
        output.join("\n")
    }
}

pub fn sanitize_arg_name(name: &str) -> String {
    name.replace(['-', '.', ':'], "_")
}
