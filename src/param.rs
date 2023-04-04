use crate::argc_value::ArgcValue;
use crate::parser::Position;
use crate::utils::{
    escape_shell_words, is_choice_value_terminate, is_default_value_terminate, to_cobol_case,
};

use anyhow::{bail, Result};
use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgAction, ArgMatches};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write;

pub const EXTRA_ARGS: &str = "_args";

pub type ParamNames = (HashMap<String, Position>, HashMap<char, Position>);

#[derive(Debug, Clone)]
pub struct ParamData {
    pub name: String,
    pub choices: Option<Vec<String>>,
    pub choices_fn: Option<String>,
    pub multiple: bool,
    pub required: bool,
    pub default: Option<String>,
    pub default_fn: Option<String>,
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
#[serde(tag = "type")]
pub enum Param {
    Flag(FlagParam),
    Option(OptionParam),
    Positional(PositionalParam),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct FlagParam {
    pub(crate) name: String,
    pub(crate) summary: String,
    pub(crate) short: Option<char>,
    pub(crate) no_long: bool,
    pub(crate) multiple: bool,
}

impl FlagParam {
    pub fn new(arg: ParamData, summary: &str, short: Option<char>, no_long: bool) -> Self {
        FlagParam {
            name: arg.name,
            short,
            no_long,
            multiple: arg.multiple,
            summary: summary.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tag_name(&self) -> &str {
        "@flag"
    }

    pub fn render(&self) -> String {
        let mut output = vec![];
        let multiple = if self.multiple { "*" } else { "" };
        if self.no_long {
            if let Some(ch) = self.short {
                output.push(format!("-{}{}", ch, multiple));
            }
        } else {
            if let Some(ch) = self.short {
                output.push(format!("-{}", ch));
            }
            output.push(format!("--{}{}", self.name, multiple));
        }
        render_summary(&mut output, &self.summary);
        output.join(" ")
    }

    pub fn build_arg(&self) -> Result<Arg> {
        let mut arg = new_arg(&self.name, &self.summary);
        if !self.no_long {
            arg = arg.long(self.name.to_string());
        }
        if let Some(s) = self.short {
            arg = arg.short(s);
        }
        if self.name == "help" {
            arg = arg.action(ArgAction::Help)
        } else if self.name == "version" {
            arg = arg.action(ArgAction::Version)
        } else if self.multiple {
            arg = arg.action(ArgAction::Count);
        } else {
            arg = arg.action(ArgAction::SetTrue);
        }
        Ok(arg)
    }

    pub fn get_arg_value(&self, matches: &ArgMatches) -> Option<ArgcValue> {
        if self.multiple {
            let count = matches.get_count(&self.name);
            return Some(ArgcValue::Single(self.name.clone(), count.to_string()));
        }
        if let Ok(Some(&true)) = matches.try_get_one::<bool>(&self.name) {
            return Some(ArgcValue::Single(self.name.clone(), "1".into()));
        };
        None
    }

    pub fn detect_conflict(&self, names: &mut ParamNames, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(&self.name, false, tag_name, names, pos)?;
        detect_short_name_conflict(&self.short, tag_name, names, pos)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct OptionParam {
    pub(crate) name: String,
    pub(crate) summary: String,
    pub(crate) short: Option<char>,
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

impl OptionParam {
    pub fn new(
        arg: ParamData,
        summary: &str,
        short: Option<char>,
        no_long: bool,
        value_names: &[&str],
    ) -> Self {
        let value_names: Vec<String> = value_names.iter().map(|v| v.to_string()).collect();
        let arg_value_names = if value_names.is_empty() {
            vec![to_cobol_case(&arg.name)]
        } else {
            value_names.iter().map(|v| to_cobol_case(v)).collect()
        };
        OptionParam {
            name: arg.name.clone(),
            summary: summary.to_string(),
            short,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tag_name(&self) -> &str {
        "@option"
    }

    pub fn render(&self) -> String {
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
        render_summary(&mut output, &self.summary);
        output.join(" ")
    }

    pub fn build_arg(&self) -> Result<Arg> {
        let mut arg = new_arg(&self.name, &self.summary);
        if !self.no_long {
            arg = arg.long(self.name.to_string())
        }
        if let Some(s) = self.short {
            arg = arg.short(s);
        }
        arg = arg.required(self.required);
        if self.arg_value_names.len() == 1 {
            arg = arg.value_name(&self.arg_value_names[0]);
        } else {
            arg = arg.value_names(&self.arg_value_names);
        }
        if self.multiple {
            let num = usize::from(self.required);
            arg = arg
                .value_delimiter(',')
                .action(ArgAction::Append)
                .num_args(num..)
        }
        if let Some(choices) = &self.choices {
            if choices.len() > 1 {
                arg = arg.value_parser(PossibleValuesParser::new(choices));
            }
        }
        if let Some(default) = self.default.as_ref() {
            arg = arg.default_value(default);
        }
        Ok(arg)
    }

    pub fn build_arg_loose(&self) -> Result<Arg> {
        let mut arg = new_arg(&self.name, &self.summary);
        if !self.no_long {
            arg = arg.long(self.name.to_string())
        }
        if let Some(s) = self.short {
            arg = arg.short(s);
        }
        if self.multiple {
            arg = arg.value_delimiter(',').action(ArgAction::Append)
        }
        if let Some(default) = self.default.as_ref() {
            arg = arg.default_value(default);
        }
        Ok(arg)
    }

    pub fn get_arg_value(&self, matches: &ArgMatches) -> Option<ArgcValue> {
        if !matches.contains_id(&self.name) {
            if let Some(default_fn) = self.default_fn.as_ref() {
                return Some(ArgcValue::SingleFn(
                    self.name.clone(),
                    default_fn.to_string(),
                ));
            }
            return None;
        }
        if self.multiple || self.arg_value_names.len() > 1 {
            let values = matches
                .get_many::<String>(&self.name)
                .map(|vals| vals.map(|v| escape_shell_words(v)).collect::<Vec<_>>())
                .unwrap_or_default();
            Some(ArgcValue::Multiple(self.name.clone(), values))
        } else {
            let value = matches
                .get_one::<String>(&self.name)
                .map(|v| escape_shell_words(v))
                .unwrap_or_default();
            Some(ArgcValue::Single(self.name.clone(), value))
        }
    }

    pub fn detect_conflict(&self, names: &mut ParamNames, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(&self.name, false, tag_name, names, pos)?;
        detect_short_name_conflict(&self.short, tag_name, names, pos)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct PositionalParam {
    pub(crate) name: String,
    pub(crate) summary: String,
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
    pub fn new(arg: ParamData, summary: &str, value_name: Option<&str>) -> Self {
        PositionalParam {
            name: arg.name.clone(),
            summary: summary.to_string(),
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

    pub fn extra() -> Self {
        PositionalParam {
            name: EXTRA_ARGS.to_string(),
            summary: "".to_string(),
            choices: None,
            choices_fn: None,
            multiple: true,
            required: false,
            default: None,
            default_fn: None,
            value_name: None,
            arg_value_name: EXTRA_ARGS.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tag_name(&self) -> &str {
        "@arg"
    }

    pub fn render(&self) -> String {
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
        render_summary(&mut output, &self.summary);
        output.join(" ")
    }

    pub fn build_arg(&self, index: usize) -> Result<Arg> {
        let mut arg = new_arg(&self.name, &self.summary);
        arg = arg
            .index(index + 1)
            .required(self.required)
            .value_name(&self.arg_value_name);
        if self.name == EXTRA_ARGS {
            arg = arg.hide(true);
        }
        if let Some(choices) = &self.choices {
            if choices.len() > 1 {
                arg = arg.value_parser(PossibleValuesParser::new(choices));
            }
        }
        if self.multiple {
            arg = arg.action(ArgAction::Append)
        }
        if let Some(default) = self.default.as_ref() {
            arg = arg.default_value(default);
        }
        Ok(arg)
    }

    pub fn get_arg_value(&self, matches: &ArgMatches) -> Option<ArgcValue> {
        if !matches.contains_id(&self.name) {
            if let Some(default_fn) = self.default_fn.as_ref() {
                return Some(ArgcValue::PositionalSingleFn(
                    self.name.clone(),
                    default_fn.to_string(),
                ));
            }
            return None;
        }
        if self.multiple {
            let values = matches
                .get_many::<String>(&self.name)
                .map(|vals| vals.map(|v| escape_shell_words(v)).collect::<Vec<_>>())
                .unwrap_or_default();
            Some(ArgcValue::PositionalMultiple(self.name.clone(), values))
        } else {
            let value = escape_shell_words(matches.get_one::<String>(&self.name).unwrap());
            Some(ArgcValue::PositionalSingle(self.name.clone(), value))
        }
    }

    pub fn detect_conflict(&self, names: &mut ParamNames, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(&self.name, true, tag_name, names, pos)
    }
}

fn render_summary(output: &mut Vec<String>, summary: &str) {
    if !summary.is_empty() {
        output.push(summary.to_string());
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

fn new_arg(name: &str, summary: &str) -> Arg {
    let mut arg = Arg::new(name.to_string());
    let title = summary.trim();
    if !title.is_empty() {
        arg = arg.help(title.to_string());
    }
    arg
}

fn detect_name_conflict(
    name: &str,
    is_positional: bool,
    tag_name: &str,
    names: &mut ParamNames,
    pos: Position,
) -> Result<()> {
    let name_desc = if is_positional {
        format!("`{}`", name)
    } else {
        format!("--{}", name)
    };
    if let Some(exist_pos) = names.0.get(name) {
        bail!(
            "{}(line {}) has {} already exists at line {}",
            tag_name,
            pos,
            name_desc,
            exist_pos,
        );
    } else {
        names.0.insert(name.to_string(), pos);
    }
    Ok(())
}

fn detect_short_name_conflict(
    short: &Option<char>,
    tag_name: &str,
    names: &mut ParamNames,
    pos: Position,
) -> Result<()> {
    if let Some(ch) = short {
        if let Some(exist_pos) = names.1.get(ch) {
            bail!(
                "{}(line {}) has -{} already exists at line {}",
                tag_name,
                pos,
                ch,
                exist_pos,
            )
        } else {
            names.1.insert(*ch, pos);
        }
    }
    Ok(())
}
