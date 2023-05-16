use crate::{
    utils::{
        escape_shell_words, is_choice_value_terminate, is_default_value_terminate, to_cobol_case,
    },
    ArgcValue,
};

use serde::Serialize;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct ParamData {
    pub(crate) name: String,
    pub(crate) choices: Option<Vec<String>>,
    pub(crate) choices_fn: Option<String>,
    pub(crate) multiple: bool,
    pub(crate) required: bool,
    pub(crate) default: Option<String>,
    pub(crate) default_fn: Option<String>,
}

impl ParamData {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            choices: None,
            choices_fn: None,
            multiple: false,
            required: false,
            default: None,
            default_fn: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct FlagOptionParam {
    pub(crate) name: String,
    pub(crate) describe: String,
    pub(crate) short: Option<char>,
    pub(crate) flag: bool,
    pub(crate) no_long: bool,
    pub(crate) choices: Option<Vec<String>>,
    pub(crate) choices_fn: Option<String>,
    pub(crate) multiple: bool,
    pub(crate) required: bool,
    pub(crate) default: Option<String>,
    pub(crate) default_fn: Option<String>,
    pub(crate) value_names: Vec<String>,
    #[serde(skip_serializing)]
    pub(crate) arg_value_names: Vec<String>,
}

impl FlagOptionParam {
    pub(crate) fn new(
        arg: ParamData,
        describe: &str,
        short: Option<char>,
        flag: bool,
        no_long: bool,
        value_names: &[&str],
    ) -> Self {
        let value_names: Vec<String> = value_names.iter().map(|v| v.to_string()).collect();
        let arg_value_names = if value_names.is_empty() {
            vec![to_cobol_case(&arg.name)]
        } else {
            value_names.iter().map(|v| to_cobol_case(v)).collect()
        };
        Self {
            name: arg.name.clone(),
            describe: describe.to_string(),
            short,
            flag,
            no_long,
            choices: arg.choices,
            choices_fn: arg.choices_fn,
            multiple: arg.multiple,
            required: arg.required,
            default: arg.default,
            default_fn: arg.default_fn,
            value_names,
            arg_value_names,
        }
    }

    pub(crate) fn is_flag(&self) -> bool {
        self.flag
    }

    pub(crate) fn is_option(&self) -> bool {
        !self.is_flag()
    }

    pub(crate) fn tag_name(&self) -> &str {
        if self.is_flag() {
            "@flag"
        } else {
            "@option"
        }
    }

    #[allow(unused)]
    pub(crate) fn render(&self) -> String {
        let mut output = vec![];
        if self.no_long {
            let name = render_name(
                &self.name,
                &self.choices,
                &self.choices_fn,
                self.multiple,
                self.required,
                &self.default,
                &self.default_fn,
            );
            output.push(format!("-{}", name));
        } else {
            if let Some(ch) = self.short {
                output.push(format!("-{}", ch));
            };
            let name = render_name(
                &self.name,
                &self.choices,
                &self.choices_fn,
                self.multiple,
                self.required,
                &self.default,
                &self.default_fn,
            );
            output.push(format!("--{}", name));
        }
        for value_name in &self.value_names {
            output.push(format!("<{}>", value_name));
        }
        if !self.describe.is_empty() {
            output.push(self.describe.clone());
        }
        output.join(" ")
    }

    pub(crate) fn render_name(&self) -> String {
        if self.no_long {
            format!("-{}", self.name)
        } else {
            format!("--{}", self.name)
        }
    }

    pub(crate) fn render_single_value(&self) -> String {
        format!("<{}>", self.arg_value_names[0])
    }

    pub(crate) fn render_name_values(&self) -> String {
        let mut output = self.render_name();
        output.push_str(&self.render_arg_values());
        output
    }

    pub(crate) fn render_body(&self) -> String {
        let mut output = match (self.no_long, self.short) {
            (true, _) => {
                format!("-{}", self.name)
            }
            (false, Some(c)) => {
                format!("-{c}, --{}", self.name)
            }
            (false, None) => {
                format!("    --{}", self.name)
            }
        };
        if self.is_flag() {
            if self.multiple {
                output.push_str("...")
            }
        } else {
            output.push_str(&self.render_arg_values());
        }
        output
    }

    pub(crate) fn render_arg_values(&self) -> String {
        if self.is_flag() {
            return String::new();
        }
        let mut output = " ".to_string();
        if self.arg_value_names.len() == 1 {
            let name: &String = &self.arg_value_names[0];
            let value = match (self.required, self.multiple) {
                (true, true) => format!("<{name}>..."),
                (false, true) => format!("[<{name}>...]"),
                (_, false) => format!("<{name}>"),
            };
            output.push_str(&value);
        } else {
            let values = self
                .arg_value_names
                .iter()
                .map(|v| format!("<{v}>"))
                .collect::<Vec<String>>();
            output.push_str(&values.join(" "));
        }
        output
    }

    pub(crate) fn render_describe(&self) -> String {
        render_describe(&self.describe, &self.default, &self.choices)
    }

    pub(crate) fn get_arg_value(&self, values: &[&[&str]]) -> Option<ArgcValue> {
        let name = self.name.clone();
        if self.flag {
            if values.is_empty() {
                None
            } else {
                Some(ArgcValue::Single(name, values.len().to_string()))
            }
        } else {
            if values.is_empty() {
                if let Some(value) = self.default.as_ref() {
                    return Some(ArgcValue::Single(name, value.clone()));
                }
                if let Some(value) = self.default_fn.as_ref() {
                    return Some(ArgcValue::SingleFn(name, value.clone()));
                }
                return None;
            }
            if self.multiple {
                let values: Vec<String> = values.iter().map(|v| must_get_first(v)).collect();
                Some(ArgcValue::Multiple(name, values))
            } else if self.values_size() > 1 {
                Some(ArgcValue::Multiple(
                    name,
                    values[0].iter().map(|v| v.to_string()).collect(),
                ))
            } else {
                Some(ArgcValue::Single(name, must_get_first(values[0])))
            }
        }
    }

    pub(crate) fn is_match(&self, name: &str) -> bool {
        self.list_names().iter().any(|v| v == name)
    }

    pub(crate) fn list_names(&self) -> Vec<String> {
        let mut output = vec![];
        if self.no_long {
            output.push(format!("-{}", self.name));
        } else {
            output.push(format!("--{}", self.name));
            if let Some(short) = self.short {
                output.push(format!("-{}", short));
            }
        }
        output
    }

    pub(crate) fn values_size(&self) -> usize {
        if self.is_flag() {
            0
        } else if self.multiple {
            9999
        } else {
            self.arg_value_names.len()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct PositionalParam {
    pub(crate) name: String,
    pub(crate) describe: String,
    pub(crate) choices: Option<Vec<String>>,
    pub(crate) choices_fn: Option<String>,
    pub(crate) multiple: bool,
    pub(crate) required: bool,
    pub(crate) default: Option<String>,
    pub(crate) default_fn: Option<String>,
    pub(crate) value_name: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) arg_value_name: String,
}

impl PositionalParam {
    pub(crate) fn new(arg: ParamData, describe: &str, value_name: Option<&str>) -> Self {
        PositionalParam {
            name: arg.name.clone(),
            describe: describe.to_string(),
            choices: arg.choices,
            choices_fn: arg.choices_fn,
            multiple: arg.multiple,
            required: arg.required,
            default: arg.default,
            default_fn: arg.default_fn,
            value_name: value_name.map(|v| v.to_string()),
            arg_value_name: value_name
                .or(Some(&arg.name))
                .map(to_cobol_case)
                .unwrap_or_default(),
        }
    }

    pub(crate) fn tag_name(&self) -> &str {
        "@arg"
    }

    #[allow(unused)]
    pub(crate) fn render(&self) -> String {
        let mut output = vec![];
        let name = render_name(
            &self.name,
            &self.choices,
            &self.choices_fn,
            self.multiple,
            self.required,
            &self.default,
            &self.default_fn,
        );
        output.push(name);
        if let Some(value_name) = self.value_name.as_ref() {
            output.push(format!("<{}>", value_name));
        }
        if !self.describe.is_empty() {
            output.push(self.describe.clone());
        }
        output.join(" ")
    }

    pub(crate) fn render_value(&self) -> String {
        let name: &String = &self.arg_value_name;
        match (self.required, self.multiple) {
            (true, true) => format!("<{name}>..."),
            (true, false) => format!("<{name}>"),
            (false, true) => format!("[{name}]..."),
            (false, false) => format!("[{name}]"),
        }
    }

    pub(crate) fn render_describe(&self) -> String {
        render_describe(&self.describe, &self.default, &self.choices)
    }

    pub(crate) fn get_arg_value(&self, values: &[&str]) -> Option<ArgcValue> {
        let name = self.name.clone();
        if values.is_empty() {
            if let Some(value) = self.default.as_ref() {
                return Some(ArgcValue::PositionalSingle(name, value.clone()));
            }
            if let Some(value) = self.default_fn.as_ref() {
                return Some(ArgcValue::PositionalSingleFn(name, value.clone()));
            }
            return None;
        }
        if self.multiple {
            let values: Vec<String> = values.iter().map(|v| v.to_string()).collect();
            Some(ArgcValue::PositionalMultiple(name, values))
        } else {
            Some(ArgcValue::PositionalSingle(name, must_get_first(values)))
        }
    }
}

fn render_name(
    name: &str,
    choices: &Option<Vec<String>>,
    choices_fn: &Option<String>,
    multiple: bool,
    required: bool,
    default: &Option<String>,
    default_fn: &Option<String>,
) -> String {
    let mut name = name.to_string();
    if let Some(choices) = choices {
        if let Some(ch) = get_modifer(required, multiple) {
            name.push(ch)
        }
        let mut prefix = String::new();
        if default.is_some() {
            prefix.push('=');
        }
        let values: Vec<String> = choices
            .iter()
            .map(|value| {
                if value.chars().any(is_choice_value_terminate) {
                    format!("\"{}\"", value)
                } else {
                    value.to_string()
                }
            })
            .collect();
        let choices_value = format!("[{}{}]", prefix, values.join("|"));
        name.push_str(&choices_value);
    } else if let Some(choices_fn) = choices_fn {
        if let Some(ch) = get_modifer(required, multiple) {
            name.push(ch)
        }
        let _ = write!(name, "[`{}`]", choices_fn);
    } else if let Some(default) = default {
        let value = if default.chars().any(is_default_value_terminate) {
            format!("\"{}\"", default)
        } else {
            default.to_string()
        };
        let _ = write!(name, "={}", value);
    } else if let Some(default_fn) = default_fn {
        let _ = write!(name, "=`{}`", default_fn);
    } else if let Some(ch) = get_modifer(required, multiple) {
        name.push(ch)
    }
    name
}

fn get_modifer(required: bool, multiple: bool) -> Option<char> {
    match (required, multiple) {
        (true, true) => Some('+'),
        (true, false) => Some('!'),
        (false, true) => Some('*'),
        (false, false) => None,
    }
}

fn render_describe(
    describe: &str,
    default: &Option<String>,
    choices: &Option<Vec<String>>,
) -> String {
    let mut output = describe.to_string();
    if let Some(default) = default.as_ref() {
        if !output.is_empty() {
            output.push(' ')
        }
        output.push_str(&format!("[default: {}]", escape_shell_words(default)));
    }
    if let Some(choices) = &choices.as_ref() {
        if !output.is_empty() {
            output.push(' ')
        }
        let values: Vec<String> = choices.iter().map(|v| escape_shell_words(v)).collect();
        output.push_str(&format!("[possible values: {}]", values.join(", ")));
    }
    output
}

fn must_get_first(value: &[&str]) -> String {
    if value.is_empty() {
        String::new()
    } else {
        value[0].to_string()
    }
}
