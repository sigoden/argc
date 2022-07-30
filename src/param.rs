use crate::cli::RetrieveValue;
use crate::parser::Position;
use crate::utils::{
    escape_shell_words, is_choice_value_terminate, is_default_value_terminate, to_cobol_case,
};

use anyhow::{bail, Result};
use clap::{Arg, ArgMatches};
use std::collections::HashMap;
use std::fmt::Write;

pub const DEFAULT_SHELL_POSITIONAL_ARGS: &str = "_args";

pub type ParamNames<'a> = (HashMap<&'a str, Position>, HashMap<char, Position>);

pub trait Param<'a> {
    fn tag_name(&'a self) -> &'static str;
    fn render(&'a self) -> String;
    fn build_arg(&'a self, index: usize) -> Result<Arg<'a>>;
    fn retrieve_value(&'a self, matches: &ArgMatches) -> Option<RetrieveValue<'a>>;
    fn detect_conflict(&'a self, names: &mut ParamNames<'a>, pos: Position) -> Result<()>;
    fn is_positional(&'a self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ParamData<'a> {
    pub name: &'a str,
    pub choices: Option<Vec<&'a str>>,
    pub multiple: bool,
    pub required: bool,
    pub default: Option<&'a str>,
}

impl<'a> ParamData<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            choices: None,
            multiple: false,
            required: false,
            default: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FlagParam<'a> {
    pub(crate) name: &'a str,
    pub(crate) summary: &'a str,
    pub(crate) short: Option<char>,
}

impl<'a> FlagParam<'a> {
    pub fn new(arg: ParamData<'a>, summary: &'a str, short: Option<char>) -> Self {
        FlagParam {
            name: arg.name,
            short,
            summary,
        }
    }
}

impl<'a> Param<'a> for FlagParam<'a> {
    fn tag_name(&'a self) -> &'static str {
        "@flag"
    }

    fn render(&'a self) -> String {
        let mut output = vec![];
        render_short(&mut output, &self.short);
        output.push(format!("--{}", self.name));
        render_summary(&mut output, self.summary);
        output.join(" ")
    }

    fn build_arg(&'a self, _index: usize) -> Result<Arg<'a>> {
        let mut arg = new_arg(self.name, self.summary);
        arg = arg.long(self.name);
        if let Some(s) = self.short {
            arg = arg.short(s);
        }
        Ok(arg)
    }

    fn retrieve_value(&'a self, matches: &ArgMatches) -> Option<RetrieveValue<'a>> {
        if !matches.is_present(self.name) {
            return None;
        }
        Some(RetrieveValue::Single(self.name, "1".to_string()))
    }

    fn detect_conflict(&self, names: &mut ParamNames<'a>, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(self.name, false, tag_name, names, pos)?;
        detect_short_name_conflict(&self.short, tag_name, names, pos)
    }

    fn is_positional(&'a self) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OptionParam<'a> {
    pub(crate) name: &'a str,
    pub(crate) summary: &'a str,
    pub(crate) short: Option<char>,
    pub(crate) value_name: Option<&'a str>,
    pub(crate) choices: Option<Vec<&'a str>>,
    pub(crate) multiple: bool,
    pub(crate) required: bool,
    pub(crate) default: Option<&'a str>,
    pub(crate) arg_value_name: String,
}

impl<'a> OptionParam<'a> {
    pub fn new(
        arg: ParamData<'a>,
        summary: &'a str,
        short: Option<char>,
        value_name: Option<&'a str>,
    ) -> Self {
        OptionParam {
            name: arg.name,
            summary,
            choices: arg.choices,
            multiple: arg.multiple,
            required: arg.required,
            default: arg.default,
            short,
            value_name,
            arg_value_name: value_name
                .or(Some(arg.name))
                .map(to_cobol_case)
                .unwrap_or_default(),
        }
    }
}

impl<'a> Param<'a> for OptionParam<'a> {
    fn tag_name(&'a self) -> &'static str {
        "@option"
    }

    fn render(&'a self) -> String {
        let mut output = vec![];
        render_short(&mut output, &self.short);
        let name = render_name(
            self.name,
            &self.choices,
            self.multiple,
            self.required,
            &self.default,
        );
        output.push(format!("--{}", name));
        if let Some(value_name) = self.value_name {
            output.push(format!("<{}>", value_name));
        }
        render_summary(&mut output, self.summary);
        output.join(" ")
    }

    fn build_arg(&'a self, _index: usize) -> Result<Arg<'a>> {
        let mut arg = new_arg(self.name, self.summary);
        arg = arg
            .long(self.name)
            .required(self.required)
            .value_name(&self.arg_value_name);
        if let Some(s) = self.short {
            arg = arg.short(s);
        }
        if self.multiple {
            arg = arg
                .multiple_values(true)
                .use_value_delimiter(true)
                .multiple_occurrences(true);
        }
        if let Some(choices) = &self.choices {
            if choices.len() > 1 {
                arg = arg.possible_values(choices);
            }
        }
        if let Some(default) = self.default {
            arg = arg.default_value(default);
        }
        Ok(arg)
    }

    fn retrieve_value(&'a self, matches: &ArgMatches) -> Option<RetrieveValue<'a>> {
        if !matches.is_present(self.name) {
            return None;
        }
        if self.multiple {
            let values = matches
                .values_of(self.name)
                .unwrap()
                .map(escape_shell_words)
                .collect();
            Some(RetrieveValue::Multiple(self.name, values))
        } else {
            let value = escape_shell_words(matches.value_of(self.name).unwrap());
            Some(RetrieveValue::Single(self.name, value))
        }
    }

    fn detect_conflict(&self, names: &mut ParamNames<'a>, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(self.name, false, tag_name, names, pos)?;
        detect_short_name_conflict(&self.short, tag_name, names, pos)
    }

    fn is_positional(&'a self) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PositionalParam<'a> {
    pub(crate) name: &'a str,
    pub(crate) summary: &'a str,
    pub(crate) choices: Option<Vec<&'a str>>,
    pub(crate) multiple: bool,
    pub(crate) required: bool,
    pub(crate) default: Option<&'a str>,
    pub(crate) arg_value_name: String,
}

impl<'a> PositionalParam<'a> {
    pub fn new(arg: ParamData<'a>, summary: &'a str) -> Self {
        PositionalParam {
            name: arg.name,
            summary,
            choices: arg.choices,
            multiple: arg.multiple,
            required: arg.required,
            default: arg.default,
            arg_value_name: to_cobol_case(arg.name),
        }
    }

    pub fn default_shell_positional() -> Self {
        PositionalParam {
            name: DEFAULT_SHELL_POSITIONAL_ARGS,
            summary: "",
            choices: None,
            multiple: true,
            required: false,
            default: None,
            arg_value_name: DEFAULT_SHELL_POSITIONAL_ARGS.to_string(),
        }
    }
}

impl<'a> Param<'a> for PositionalParam<'a> {
    fn tag_name(&'a self) -> &'static str {
        "@arg"
    }

    fn render(&'a self) -> String {
        let mut output = vec![];
        let name = render_name(
            self.name,
            &self.choices,
            self.multiple,
            self.required,
            &self.default,
        );
        output.push(name);
        render_summary(&mut output, self.summary);
        output.join(" ")
    }

    fn build_arg(&'a self, index: usize) -> Result<Arg<'a>> {
        let mut arg = new_arg(self.name, self.summary);
        arg = arg
            .index(index + 1)
            .required(self.required)
            .value_name(&self.arg_value_name);
        if self.name == DEFAULT_SHELL_POSITIONAL_ARGS {
            arg = arg.hide(true);
        }
        if let Some(choices) = &self.choices {
            if choices.len() > 1 {
                arg = arg.possible_values(choices);
            }
        }
        if self.multiple {
            arg = arg.multiple_values(true)
        }
        if let Some(default) = self.default {
            arg = arg.default_value(default);
        }
        Ok(arg)
    }

    fn retrieve_value(&'a self, matches: &ArgMatches) -> Option<RetrieveValue<'a>> {
        if !matches.is_present(self.name) {
            return None;
        }
        if self.multiple {
            let values = matches
                .values_of(self.name)
                .unwrap()
                .map(escape_shell_words)
                .collect();
            Some(RetrieveValue::PositionalMultiple(self.name, values))
        } else {
            let value = escape_shell_words(matches.value_of(self.name).unwrap());
            Some(RetrieveValue::PositionalSingle(self.name, value))
        }
    }

    fn detect_conflict(&self, names: &mut ParamNames<'a>, pos: Position) -> Result<()> {
        let tag_name = self.tag_name();
        detect_name_conflict(self.name, true, tag_name, names, pos)
    }

    fn is_positional(&'a self) -> bool {
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

fn render_name<'a>(
    name: &'a str,
    choices: &Option<Vec<&'a str>>,
    multiple: bool,
    required: bool,
    default: &Option<&'a str>,
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
    } else if let Some(default) = default {
        let value = if default.chars().any(is_default_value_terminate) {
            format!("\"{}\"", default)
        } else {
            default.to_string()
        };
        let _ = write!(name, "={}", value);
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

fn new_arg<'a>(name: &'a str, summary: &'a str) -> Arg<'a> {
    let mut arg = Arg::new(name);
    let title = summary.trim();
    if !title.is_empty() {
        arg = arg.help(title);
    }
    arg
}

fn detect_name_conflict<'a>(
    name: &'a str,
    is_positional: bool,
    tag_name: &str,
    names: &mut ParamNames<'a>,
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
        names.0.insert(name, pos);
    }
    Ok(())
}

fn detect_short_name_conflict<'a>(
    short: &Option<char>,
    tag_name: &str,
    names: &mut ParamNames<'a>,
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
