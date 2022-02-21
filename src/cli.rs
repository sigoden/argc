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
    pub fn build(self, name: &'a str) -> Command<'a> {
        let mut app = Command::new(name);
        let mut app_positional_index: usize = 0;
        let mut subcmd: Option<Command> = None;
        let mut subcmd_positional_index: usize = 0;
        for Event { data, .. } in self.events {
            match data {
                EventData::Describe(value) => {
                    if let Some(_) = subcmd {
                    } else {
                        app = app.about(value);
                    }
                }
                EventData::Version(value) => {
                    if let Some(_) = subcmd {
                    } else {
                        app = app.version(value);
                    }
                }
                EventData::Cmd(value) => {
                    let mut cmd = Command::default();
                    if value.len() > 0 {
                        cmd = cmd.about(value);
                    }
                    subcmd = Some(cmd);
                    subcmd_positional_index = 0;
                }
                EventData::Arg(arg) => {
                    if let Some(cmd) = subcmd {
                        let cmd = cmd.arg(arg.build(subcmd_positional_index));
                        subcmd = Some(cmd);
                        subcmd_positional_index += 1;
                    } else {
                        app = app.arg(arg.build(app_positional_index));
                        app_positional_index += 1;
                    }
                }
                EventData::Func(name) => {
                    let mut cmd = subcmd.take().unwrap_or_default();
                    cmd = cmd.name(name);
                    app = app.subcommand(cmd);
                }
                EventData::Unknown(_) => {}
            }
        }
        app
    }

    /// Generate eval script
    pub fn eval(matches: ArgMatches) -> String {
        todo!()
    }
}
