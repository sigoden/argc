use crate::parser::{parse, Event, EventData};
use crate::Result;
use clap::{Arg, Command};

#[derive(Debug, PartialEq, Clone)]
pub struct ArgData<'a> {
    pub name: &'a str,
    pub title: Option<&'a str>,
    pub arg_type: ArgType,
    pub short: Option<char>,
    pub choices: Option<Vec<&'a str>>,
    pub multiple: bool,
    pub required: bool,
    pub default: Option<&'a str>,
}

impl<'a> ArgData<'a> {
    pub fn new(name: &'a str) -> Self {
        ArgData {
            name,
            title: None,
            arg_type: ArgType::String,
            short: None,
            choices: None,
            multiple: false,
            required: false,
            default: None,
        }
    }
    pub fn build(self, index: Option<usize>) -> Result<Arg<'a>> {
        let mut arg = Arg::new(self.name);
        if let Some(idx) = index {
            arg = arg.index(idx);
        } else {
            arg = arg.long(self.name);
        }
        if let Some(title) = self.title {
            let title = title.trim();
            if title.len() > 0 {
                arg = arg.help(title);
            }
        }
        if let Some(short) = self.short {
            arg = arg.short(short);
        }
        if let Some(choices) = self.choices {
            if choices.len() > 1 {
                arg = arg.possible_values(choices);
            }
        }
        if self.multiple {
            arg = arg.multiple_values(true);
        }
        if self.required {
            arg = arg.required(true);
        }
        if let Some(default) = self.default {
            arg = arg.default_value(default);
        }
        match self.arg_type {
            ArgType::Boolean => {
            }
            ArgType::String => {
                arg = arg.takes_value(true);
            }
            ArgType::Number => {
                arg = arg.takes_value(true);
            }
        }
        Ok(arg)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArgType {
    Boolean,
    String,
    Number,
}

/// Parse shell script to generate command
pub fn build(source: &str) -> Result<Command> {
    let mut main = Command::default();
    let mut subcmd: Option<Command> = None;
    let mut subcmd_param_index: usize = 0;
    let events = parse(source)?;
    for Event { data, .. } in events {
        match data {
            EventData::Title(value) => {
                main = main.about(value);
            }
            EventData::Command(value) => {
                let mut cmd = Command::default();
                if value.len() > 0 {
                    cmd = cmd.about(value);
                }
                subcmd = Some(cmd);
            }
            EventData::OptionArg(arg) => {
                if let Some(cmd) = subcmd {
                    let cmd = cmd.arg(arg.build(None)?);
                    subcmd = Some(cmd);
                } else {
                    main = main.arg(arg.build(None)?);
                }
            }
            EventData::ParamArg(arg) => {
                if let Some(cmd) = subcmd {
                    let cmd = cmd.arg(arg.build(Some(subcmd_param_index))?);
                    subcmd_param_index += 1;
                    subcmd = Some(cmd);
                }
            }
            EventData::Func(name) => {
                let mut cmd = subcmd.unwrap_or_default();
                cmd = cmd.name(name);
                subcmd = Some(cmd)
            }
            EventData::Unknown(_) => {}
        }
    }
    Ok(main)
}
