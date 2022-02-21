use crate::arg::ArgData;
use crate::parser::{parse, Event, EventData};
use crate::Result;
use clap::Command;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Cli<'a> {
    events: Vec<Event<'a>>,
}

impl<'a> Cli<'a> {
    fn new(events: Vec<Event<'a>>) -> Self {
        Self { events }
    }
    fn build(
        &'a self,
        name: &'a str,
    ) -> (
        Command<'a>,
        Vec<ArgData<'a>>,
        HashMap<&'a str, Vec<ArgData<'a>>>,
    ) {
        let mut app_args: Vec<ArgData> = vec![];
        let mut app = Command::new(name);
        let mut app_positional_index: usize = 0;
        let mut cmds_args: HashMap<&'a str, Vec<ArgData<'a>>> = Default::default();
        let mut subcmd: Option<Command> = None;
        let mut subcmd_args: Vec<ArgData> = vec![];
        let mut subcmd_positional_index: usize = 0;
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
                    if let Some(cmd) = subcmd.take() {
                        app = app.subcommand(cmd.name(*name));
                        cmds_args.insert(*name, subcmd_args);
                        subcmd_args = vec![];
                    }
                }
                EventData::Unknown(_) => {}
            }
        }
        (app, app_args, cmds_args)
    }
}

/// Run script with arguments, returns (stdout, stderr)
pub fn eval<'a>(source: &'a str, args: &[&'a str]) -> Result<(Option<String>, Option<String>)> {
    let events = parse(source)?;
    let cli = Cli::new(events);
    let (app, app_args, cmds_args) = cli.build(args[0]);
    let res = app.try_get_matches_from(args);
    match res {
        Ok(matches) => {
            let mut output = String::new();
            for arg_data in app_args {
                if let Some(arg_value) = arg_data.retrive(&matches) {
                    output.push_str(&arg_value);
                }
            }
            if let Some((name, cmd_matches)) = matches.subcommand() {
                let cmd_args = cmds_args.get(name).unwrap();
                for arg_data in cmd_args {
                    if let Some(arg_value) = arg_data.retrive(&cmd_matches) {
                        output.push_str(&arg_value);
                    }
                }
                output.push_str(name);
            } else {
                output.push_str("main");
            }
            Ok((Some(output), None))
        }
        Err(err) => Ok((Some(format!("exit 1")), Some(err.to_string()))),
    }
}
