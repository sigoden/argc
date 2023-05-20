use crate::utils::{escape_shell_words, hyphens_to_underscores};

pub const VARIABLE_PREFIX: &str = "argc";

#[derive(Debug, PartialEq, Eq)]
pub enum ArgcValue {
    Single(String, String),
    SingleFn(String, String),
    Multiple(String, Vec<String>),
    PositionalSingle(String, String),
    PositionalSingleFn(String, String),
    PositionalMultiple(String, Vec<String>),
    ExtraPositionalMultiple(Vec<String>),
    CmdFn(String),
    ParamFn(String),
    Error((String, i32)),
}

impl ArgcValue {
    pub fn to_shell(values: Vec<Self>) -> String {
        let mut variables = vec![];
        let mut last = String::new();
        let mut call = String::new();
        let mut positional_args = vec![];
        for value in values {
            match value {
                ArgcValue::Single(name, value) => {
                    variables.push(format!(
                        "{}_{}={}",
                        VARIABLE_PREFIX,
                        hyphens_to_underscores(&name),
                        escape_shell_words(&value)
                    ));
                }
                ArgcValue::SingleFn(name, fn_name) => {
                    variables.push(format!(
                        "{}_{}=`{}`",
                        VARIABLE_PREFIX,
                        hyphens_to_underscores(&name),
                        fn_name,
                    ));
                }
                ArgcValue::Multiple(name, values) => {
                    variables.push(format!(
                        "{}_{}=( {} )",
                        VARIABLE_PREFIX,
                        name,
                        values
                            .iter()
                            .map(|v| escape_shell_words(v))
                            .collect::<Vec<String>>()
                            .join(" ")
                    ));
                }
                ArgcValue::PositionalSingle(name, value) => {
                    let value = escape_shell_words(&value);
                    variables.push(format!(
                        "{}_{}={}",
                        VARIABLE_PREFIX,
                        hyphens_to_underscores(&name),
                        &value
                    ));
                    positional_args.push(value);
                }
                ArgcValue::PositionalSingleFn(name, fn_name) => {
                    variables.push(format!(
                        "{}_{}=`{}`",
                        VARIABLE_PREFIX,
                        hyphens_to_underscores(&name),
                        &fn_name
                    ));
                    positional_args.push(format!("`{}`", fn_name));
                }
                ArgcValue::PositionalMultiple(name, values) => {
                    let values = values
                        .iter()
                        .map(|v| escape_shell_words(v))
                        .collect::<Vec<String>>();
                    variables.push(format!(
                        "{}_{}=( {} )",
                        VARIABLE_PREFIX,
                        hyphens_to_underscores(&name),
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
                ArgcValue::CmdFn(name) => {
                    if positional_args.is_empty() {
                        last = name.to_string();
                    } else {
                        last = format!("{} {}", name, positional_args.join(" "));
                    }
                    call = name.clone();
                }
                ArgcValue::ParamFn(name) => {
                    if positional_args.is_empty() {
                        last = format!("{name};exit;");
                    } else {
                        last = format!("{} {};exit;", name, positional_args.join(" "));
                    }
                    call = name.clone();
                }
                ArgcValue::Error((error, exit)) => {
                    return format!("cat >&2 <<-'EOF' \n{}\nEOF\nexit {}", error, exit)
                }
            }
        }

        variables.push(format!(
            "{}__args=( {} )",
            VARIABLE_PREFIX,
            positional_args.join(" ")
        ));

        if !call.is_empty() {
            variables.push(format!("{}__fn={}", VARIABLE_PREFIX, call));
        }

        if !last.is_empty() {
            variables.push(last);
        }

        variables.join("\n")
    }

    pub fn is_cmd_fn(&self) -> bool {
        matches!(self, Self::CmdFn(_))
    }
}
