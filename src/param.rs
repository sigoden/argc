use crate::argc_value::ArgcValue;
use crate::parser::Position;
use crate::utils::{
    escape_shell_words, is_choice_value_terminate, is_default_value_terminate, to_cobol_case,
};

use anyhow::{bail, Result};
use clap::builder::PossibleValuesParser;
use clap::{Arg, ArgAction, ArgMatches};
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

pub trait Param {
    fn name(&self) -> &str;
    fn tag_name(&self) -> &str;
    fn render(&self) -> String;
    fn build_arg(&self, index: usize) -> Result<Arg>;
    fn get_arg_value(&self, matches: &ArgMatches) -> Option<ArgcValue>;
    fn detect_conflict(&self, names: &mut ParamNames, pos: Position) -> Result<()>;
    fn is_positional(&self) -> bool;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FlagParam {
    pub(crate) name: String,
    pub(crate) summary: String,
    pub(crate) short: Option<char>,
}

impl FlagParam {
    pub fn new(arg: ParamData, summary: &str, short: Option<char>) -> Self {
        FlagParam {
            name: arg.name,
            short,
            summary: summary.to_string(),
        }
    }
}

impl Param for FlagParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn tag_name(&self) -> &str {
        "@flag"
    }

    fn render(&self) -> String {
        let mut output = vec![];
        render_short(&mut output, &self.short);
        output.push(format!("--{}", self.name));
        render_summary(&mut output, &self.summary);
        output.join(" ")
    }

    fn build_arg(&self, _index: usize) -> Result<Arg> {
        let mut arg = new_arg(&self.name, &self.summary);
        arg = arg.long(self.name.to_string()).action(ArgAction::SetTrue);
        if let Some(s) = self.short {
            arg = arg.short(s);
        }
        Ok(arg)
    }

    fn get_arg_value(&self, matches: &ArgMatches) -> Option<ArgcValue> {
        if matches.get_flag(&self.name) {
            Some(ArgcValue::Single(self.name.clone(), "1".to_string()))
        } else {
            None
        }
    }

    fn detect_conflict(&self, names: &mut ParamNames, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(&self.name, false, tag_name, names, pos)?;
        detect_short_name_conflict(&self.short, tag_name, names, pos)
    }

    fn is_positional(&self) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OptionParam {
    pub(crate) name: String,
    pub(crate) summary: String,
    pub(crate) short: Option<char>,
    pub(crate) value_name: Option<String>,
    pub(crate) choices: Option<Vec<String>>,
    pub(crate) choices_fn: Option<String>,
    pub(crate) multiple: bool,
    pub(crate) required: bool,
    pub(crate) default: Option<String>,
    pub(crate) default_fn: Option<String>,
    pub(crate) arg_value_name: String,
}

impl OptionParam {
    pub fn new(
        arg: ParamData,
        summary: &str,
        short: Option<char>,
        value_name: Option<&str>,
    ) -> Self {
        OptionParam {
            name: arg.name.clone(),
            summary: summary.to_string(),
            choices: arg.choices,
            choices_fn: arg.choices_fn,
            multiple: arg.multiple,
            required: arg.required,
            default: arg.default,
            default_fn: arg.default_fn,
            short,
            value_name: value_name.map(|v| v.to_string()),
            arg_value_name: value_name
                .or(Some(&arg.name))
                .map(to_cobol_case)
                .unwrap_or_default(),
        }
    }
}

impl Param for OptionParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn tag_name(&self) -> &str {
        "@option"
    }

    fn render(&self) -> String {
        let mut output = vec![];
        render_short(&mut output, &self.short);
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
        if let Some(value_name) = self.value_name.as_ref() {
            output.push(format!("<{}>", value_name));
        }
        render_summary(&mut output, &self.summary);
        output.join(" ")
    }

    fn build_arg(&self, _index: usize) -> Result<Arg> {
        let mut arg = new_arg(&self.name, &self.summary);
        arg = arg
            .long(self.name.to_string())
            .required(self.required)
            .value_name(&self.arg_value_name);
        if let Some(s) = self.short {
            arg = arg.short(s);
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

    fn get_arg_value(&self, matches: &ArgMatches) -> Option<ArgcValue> {
        if !matches.contains_id(&self.name) {
            if let Some(default_fn) = self.default_fn.as_ref() {
                return Some(ArgcValue::SingleFn(
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
            Some(ArgcValue::Multiple(self.name.clone(), values))
        } else {
            let value = escape_shell_words(matches.get_one::<String>(&self.name).unwrap());
            Some(ArgcValue::Single(self.name.clone(), value))
        }
    }

    fn detect_conflict(&self, names: &mut ParamNames, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(&self.name, false, tag_name, names, pos)?;
        detect_short_name_conflict(&self.short, tag_name, names, pos)
    }

    fn is_positional(&self) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PositionalParam {
    pub(crate) name: String,
    pub(crate) value_name: Option<String>,
    pub(crate) summary: String,
    pub(crate) choices: Option<Vec<String>>,
    pub(crate) choices_fn: Option<String>,
    pub(crate) multiple: bool,
    pub(crate) required: bool,
    pub(crate) default: Option<String>,
    pub(crate) default_fn: Option<String>,
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
}

impl Param for PositionalParam {
    fn name(&self) -> &str {
        &self.name
    }

    fn tag_name(&self) -> &str {
        "@arg"
    }

    fn render(&self) -> String {
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

    fn build_arg(&self, index: usize) -> Result<Arg> {
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

    fn get_arg_value(&self, matches: &ArgMatches) -> Option<ArgcValue> {
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

    fn detect_conflict(&self, names: &mut ParamNames, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(&self.name, true, tag_name, names, pos)
    }

    fn is_positional(&self) -> bool {
        true
    }
}

fn render_short(output: &mut Vec<String>, short: &Option<char>) {
    if let Some(ch) = short {
        output.push(format!("-{}", ch));
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
        if required {
            name.push('!')
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
        if required {
            name.push('!')
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
    } else if let Some(ch) = match (required, multiple) {
        (true, true) => Some('+'),
        (true, false) => Some('!'),
        (false, true) => Some('*'),
        (false, false) => None,
    } {
        name.push(ch)
    }
    name
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
