use crate::utils::hyphens_to_underscores;

pub const VARIABLE_PREFIX: &str = "argc";

#[derive(Debug, PartialEq, Eq)]
pub enum ArgcValue {
    Single(String, String),
    SingleFn(String, String),
    Multiple(String, Vec<String>),
    PositionalSingle(String, String),
    PositionalSingleFn(String, String),
    PositionalMultiple(String, Vec<String>),
    CmdFnName(String),
    ParamFnName(String),
}

impl ArgcValue {
    pub fn to_shell(values: Vec<Self>) -> String {
        let mut variables = vec![];
        let mut positional_args = vec![];
        for value in values {
            match value {
                ArgcValue::Single(name, value) => {
                    variables.push(format!(
                        "{}_{}={}",
                        VARIABLE_PREFIX,
                        hyphens_to_underscores(&name),
                        value
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
                        values.join(" ")
                    ));
                }
                ArgcValue::PositionalSingle(name, value) => {
                    variables.push(format!(
                        "{}_{}={}",
                        VARIABLE_PREFIX,
                        hyphens_to_underscores(&name),
                        &value
                    ));
                    positional_args.push(value.to_string());
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
                    variables.push(format!(
                        "{}_{}=( {} )",
                        VARIABLE_PREFIX,
                        hyphens_to_underscores(&name),
                        values.join(" ")
                    ));
                    positional_args.extend(values);
                }
                ArgcValue::CmdFnName(name) => {
                    if positional_args.is_empty() {
                        variables.push(name.to_string());
                    } else {
                        variables.push(format!("{} {}", name, positional_args.join(" ")));
                    }
                }
                ArgcValue::ParamFnName(name) => {
                    variables.push(format!("{name};exit;"));
                }
            }
        }
        variables.join("\n")
    }
}
