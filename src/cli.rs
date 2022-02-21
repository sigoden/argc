use std::collections::HashMap;

use crate::arg::ArgData;
use crate::parser::{parse, Event, EventData};
use crate::Result;
use clap::Command;

#[derive(Debug, Default)]
pub struct Cli<'a> {
    events: Vec<Event<'a>>,
    args: Vec<ArgData<'a>>,
    cmds: HashMap<&'a str, Vec<ArgData<'a>>>,
}

impl<'a> Cli<'a> {
    /// Parse shell script to generate command
    fn build(&'a mut self, name: &'a str) -> Command<'a> {
        let mut app_args: Vec<ArgData> = vec![];
        let mut app = Command::new(name);
        let mut app_positional_index: usize = 0;
        let mut subcmd: Option<Command> = None;
        let mut subcmd_positional_index: usize = 0;
        let mut subcmd_args: Vec<ArgData> = vec![];
        for Event { data, .. } in &self.events {
            match data {
                EventData::Describe(value) => {
                    if let Some(_) = subcmd {
                    } else {
                        app = app.about(*value);
                    }
                }
                EventData::Version(value) => {
                    if let Some(_) = subcmd {
                    } else {
                        app = app.version(*value);
                    }
                }
                EventData::Cmd(value) => {
                    let mut cmd = Command::default();
                    if value.len() > 0 {
                        cmd = cmd.about(*value);
                    }
                    subcmd = Some(cmd);
                    subcmd_positional_index = 0;
                }
                EventData::Arg(arg) => {
                    if let Some(cmd) = subcmd {
                        let cmd = cmd.arg(arg.build(subcmd_positional_index));
                        subcmd = Some(cmd);
                        subcmd_positional_index += 1;
                        subcmd_args.push(arg.clone());
                    } else {
                        app = app.arg(arg.build(app_positional_index));
                        app_positional_index += 1;
                        app_args.push(arg.clone());
                    }
                }
                EventData::Func(name) => {
                    let mut cmd = subcmd.take().unwrap_or_default();
                    cmd = cmd.name(*name);
                    app = app.subcommand(cmd);
                    self.cmds.insert(*name, subcmd_args);
                    subcmd_args = vec![];
                }
                EventData::Unknown(_) => {}
            }
        }
        self.args = app_args;
        app
    }
}

/// Run script with arguments, returns (stdout, stderr)
pub fn eval<'a>(source: &'a str, args: &[&'a str]) -> Result<(String, String)> {
    let events = parse(source)?;
    let mut cli = Cli::default();
    cli.events = events;
    let app = cli.build(args[0]);
    let res = app.try_get_matches_from(args);
    match res {
        Ok(_matches) => {
            todo!()
        }
        Err(err) => Ok((format!("exit 1"), err.to_string())),
    }
}
