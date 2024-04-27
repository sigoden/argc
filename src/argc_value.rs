use indexmap::IndexMap;

#[cfg(feature = "eval-bash")]
use crate::utils::{
    argc_var_name, escape_shell_words, expand_dotenv, AFTER_HOOK, ARGC_REQUIRE_TOOLS, BEFORE_HOOK,
    VARIABLE_PREFIX,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ArgcValue {
    Single(String, String),
    SingleFn(String, String),
    Multiple(String, Vec<String>),
    Map(String, IndexMap<String, Vec<String>>),
    PositionalSingle(String, String),
    PositionalSingleFn(String, String),
    PositionalMultiple(String, Vec<String>),
    ExtraPositionalMultiple(Vec<String>),
    Env(String, String),
    EnvFn(String, String),
    Hook((bool, bool)),
    Dotenv(String),
    RequireTools(Vec<String>),
    CommandFn(String),
    ParamFn(String),
    Error((String, i32)),
}

#[cfg(feature = "eval-bash")]
impl ArgcValue {
    pub fn to_bash(values: &[Self]) -> String {
        let mut list = vec![];
        let mut last = String::new();
        let mut exit = false;
        let mut positional_args = vec![];
        let mut require_tools = vec![];
        let (mut before_hook, mut after_hook) = (false, false);
        for value in values {
            match value {
                ArgcValue::Single(id, value) => {
                    list.push(format!(
                        "{}={}",
                        argc_var_name(id),
                        escape_shell_words(value)
                    ));
                }
                ArgcValue::SingleFn(id, fn_name) => {
                    list.push(format!("{}=`{}`", argc_var_name(id), fn_name,));
                }
                ArgcValue::Multiple(id, values) => {
                    list.push(format!(
                        "{}=( {} )",
                        argc_var_name(id),
                        values
                            .iter()
                            .map(|v| escape_shell_words(v))
                            .collect::<Vec<String>>()
                            .join(" ")
                    ));
                }
                ArgcValue::Map(id, map) => {
                    let var_name = argc_var_name(id);
                    list.push(format!("declare -A {var_name}"));
                    list.extend(map.iter().map(|(k, v)| {
                        let v = v
                            .iter()
                            .map(|x| escape_shell_words(x))
                            .collect::<Vec<String>>()
                            .join("|");
                        format!(r#"{var_name}["{k}"]={v}"#)
                    }));
                }
                ArgcValue::PositionalSingle(id, value) => {
                    let value = escape_shell_words(value);
                    list.push(format!("{}={}", argc_var_name(id), &value));
                    positional_args.push(value);
                }
                ArgcValue::PositionalSingleFn(id, fn_name) => {
                    list.push(format!("{}=`{}`", argc_var_name(id), &fn_name));
                    positional_args.push(format!("`{}`", fn_name));
                }
                ArgcValue::PositionalMultiple(id, values) => {
                    let values = values
                        .iter()
                        .map(|v| escape_shell_words(v))
                        .collect::<Vec<String>>();
                    list.push(format!("{}=( {} )", argc_var_name(id), values.join(" ")));
                    positional_args.extend(values);
                }
                ArgcValue::ExtraPositionalMultiple(values) => {
                    let values = values
                        .iter()
                        .map(|v| escape_shell_words(v))
                        .collect::<Vec<String>>();
                    positional_args.extend(values);
                }
                ArgcValue::Env(name, value) => {
                    list.push(format!("export {}={}", name, escape_shell_words(value)));
                }
                ArgcValue::EnvFn(id, fn_name) => {
                    list.push(format!("export {}=`{}`", id, fn_name,));
                }
                ArgcValue::Hook((before, after)) => {
                    if *before {
                        before_hook = *before;
                    }
                    if *after {
                        after_hook = *after;
                    }
                }
                ArgcValue::Dotenv(value) => {
                    list.push(expand_dotenv(value));
                }
                ArgcValue::RequireTools(tools) => {
                    require_tools = tools.to_vec();
                }
                ArgcValue::CommandFn(name) => {
                    if positional_args.is_empty() {
                        last = name.to_string();
                    } else {
                        last = format!("{} {}", name, positional_args.join(" "));
                    }
                    list.push(format!("{}_fn={}", VARIABLE_PREFIX, name));
                }
                ArgcValue::ParamFn(name) => {
                    if positional_args.is_empty() {
                        last = name.clone();
                    } else {
                        last = format!("{} {}", name, positional_args.join(" "));
                    }
                    exit = true;
                }
                ArgcValue::Error((error, exit)) => {
                    return format!("command cat >&2 <<-'EOF' \n{}\nEOF\nexit {}", error, exit)
                }
            }
        }

        list.push(format!(
            "{}_positionals=( {} )",
            VARIABLE_PREFIX,
            positional_args.join(" ")
        ));
        if !require_tools.is_empty() {
            let tools = require_tools
                .iter()
                .map(|v| escape_shell_words(v))
                .collect::<Vec<_>>()
                .join(" ");
            list.push(format!(
                "\n{ARGC_REQUIRE_TOOLS}\n_argc_require_tools {tools}\n"
            ));
        }
        if before_hook {
            list.push(BEFORE_HOOK.to_string())
        }
        if !last.is_empty() {
            list.push(last);
            if after_hook {
                list.push(AFTER_HOOK.to_string())
            }
        }
        if exit {
            list.push("exit".to_string());
        }
        list.join("\n")
    }
}
