use crate::parser::{parse, Event, EventData};
use crate::Result;
use clap::{Arg, Command};

#[derive(Debug, Default)]
pub struct Cli<'a> {
    pub title: Option<&'a str>,
    pub options: Vec<ArgData<'a>>,
    pub commands: Vec<SubCommand<'a>>,
}

impl<'a> Cli<'a> {
    pub fn parse(source: &'a str) -> Result<Self> {
        CliParser::parse(source)
    }
    pub fn build(self) -> Result<Command<'a>> {
        let mut app = Command::default();
        for arg_data in self.options {
            app = app.arg(arg_data.build()?);
        }
        for cmd in self.commands {
            app = app.subcommand(cmd.build()?)
        }
        Ok(app)
    }
}

#[derive(Debug, Default)]
pub struct SubCommand<'a> {
    pub name: Option<&'a str>,
    pub title: Option<&'a str>,
    pub options: Vec<ArgData<'a>>,
    pub params: Vec<ArgData<'a>>,
}

impl<'a> SubCommand<'a> {
    fn build(self) -> Result<Command<'a>> {
        todo!()
    }
}

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
    pub fn build(self) -> Result<Arg<'a>> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArgType {
    String,
    Boolean,
    Number,
}

struct CliParser<'a> {
    cli: Cli<'a>,
    subcmd: Option<SubCommand<'a>>,
}

impl<'a> Default for CliParser<'a> {
    fn default() -> Self {
        Self {
            cli: Cli::default(),
            subcmd: None,
        }
    }
}

impl<'a> CliParser<'a> {
    /// Create app from bash script
    pub fn parse(source: &'a str) -> Result<Cli<'a>> {
        let mut builder: Self = Default::default();
        let events = parse(source)?;
        for Event { data, .. } in events {
            match data {
                EventData::Title(value) => {
                    builder.cli.title = Some(value);
                }
                EventData::Command(value) => {
                    let mut subcmd = SubCommand::default();
                    if value.len() > 0 {
                        subcmd.title = Some(value);
                    }
                    builder.subcmd = Some(subcmd);
                }
                EventData::OptionArg(arg) => {
                    if let Some(cmd) = builder.subcmd.as_mut() {
                        cmd.options.push(arg);
                    } else {
                        builder.cli.options.push(arg);
                    }
                }
                EventData::ParamArg(arg) => {
                    if let Some(cmd) = builder.subcmd.as_mut() {
                        cmd.params.push(arg);
                    }
                }
                EventData::Func(name) => {
                    let maybe_subcmd = builder.subcmd.take();
                    let mut subcmd = maybe_subcmd.unwrap_or_default();
                    subcmd.name = Some(name);
                    builder.cli.commands.push(subcmd);
                }
                EventData::Unknown(_) => {
                    // todo: warning
                }
            }
        }
        Ok(builder.cli)
    }
}
