use crate::parser::{parse, Event, EventData};
use crate::Result;
use clap::{ArgMatches, Command};

pub struct Cli<'a> {
    events: Vec<Event<'a>>,
}

impl<'a> Cli<'a> {
    /// Load from content of shell script
    pub fn from_str(source: &'a str) -> Result<Self> {
        let events = parse(source)?;
        Ok(Self { events })
    }

    /// Parse shell script to generate command
    pub fn build(self) -> Result<Command<'a>> {
        let mut main = Command::default();
        let mut subcmd: Option<Command> = None;
        for Event { data, .. } in self.events {
            match data {
                EventData::Description(value) => {
                    if let Some(_) = subcmd {
                    } else {
                        main = main.about(value);
                    }
                }
                EventData::Cmd(value) => {
                    let mut cmd = Command::default();
                    if value.len() > 0 {
                        cmd = cmd.about(value);
                    }
                    subcmd = Some(cmd);
                }
                EventData::Arg(arg) => {
                    if let Some(cmd) = subcmd {
                        let cmd = cmd.arg(arg.build(None)?);
                        subcmd = Some(cmd);
                    } else {
                        main = main.arg(arg.build(None)?);
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

    /// Generate eval script
    pub fn eval(matches: ArgMatches) -> String {
        todo!()
    }
}
