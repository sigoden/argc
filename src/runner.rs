use crate::parser::{parse, ArgData, Event, EventData};
use crate::Result;
use clap::{Arg, ArgMatches, Command};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::ops::Deref;

const ENTRYPOINT: &'static str = "main";

/// Run script with arguments, returns (stdout, stderr)
pub fn run<'a>(source: &'a str, args: &[&'a str]) -> Result<(Option<String>, Option<String>)> {
    let events = parse(source)?;
    let name = args[0];
    let (cmd, has_main) = Cmd::create(&events);
    let command = cmd.build(name);
    let res = command.try_get_matches_from(args);
    match res {
        Ok(matches) => {
            let mut output = cmd.retrive(&matches);
            if matches.subcommand_name().is_none() && has_main {
                output.push_str(ENTRYPOINT)
            }
            Ok((Some(output), None))
        }
        Err(err) => Ok((Some(format!("exit 1")), Some(err.to_string()))),
    }
}

#[derive(Debug, Default)]
struct Cmd<'a> {
    name: Option<&'a str>,
    author: Option<&'a str>,
    describe: Option<&'a str>,
    version: Option<&'a str>,
    pos_index: usize,
    root: bool,
    args: Vec<WrapArgData<'a>>,
    subcmds: HashMap<&'a str, Cmd<'a>>,
}

impl<'a> Cmd<'a> {
    fn create(events: &'a [Event]) -> (Self, bool) {
        let mut maybe_subcmd: Option<Cmd> = None;
        let mut rootcmd = Cmd::default();
        let mut has_main = false;
        rootcmd.root = true;
        for Event { data, .. } in events {
            match data {
                EventData::Describe(value) => {
                    if let Some(_) = maybe_subcmd {
                    } else {
                        rootcmd.describe = Some(*value);
                    }
                }
                EventData::Version(value) => {
                    if let Some(_) = maybe_subcmd {
                    } else {
                        rootcmd.version = Some(*value);
                    }
                }
                EventData::Author(value) => {
                    if let Some(_) = maybe_subcmd {
                    } else {
                        rootcmd.author = Some(*value);
                    }
                }
                EventData::Cmd(value) => {
                    let mut cmd = Cmd::default();
                    if value.len() > 0 {
                        cmd.describe = Some(*value);
                    }
                    maybe_subcmd = Some(cmd);
                    if *value == ENTRYPOINT {
                        has_main = true;
                    }
                }
                EventData::Arg(arg_data) => {
                    if let Some(cmd) = &mut maybe_subcmd {
                        let arg_data = WrapArgData::new(arg_data, cmd.pos_index);
                        cmd.args.push(arg_data);
                        cmd.pos_index += 1;
                    } else {
                        let arg_data = WrapArgData::new(arg_data, rootcmd.pos_index);
                        rootcmd.args.push(arg_data);
                        rootcmd.pos_index += 1;
                    }
                }
                EventData::Func(name) => {
                    if let Some(mut cmd) = maybe_subcmd.take() {
                        cmd.name = Some(name);
                        rootcmd.subcmds.insert(*name, cmd);
                    }
                }
                EventData::Unknown(_) => {}
            }
        }
        (rootcmd, has_main)
    }
    fn build(&'a self, name: &'a str) -> Command<'a> {
        let mut rootcmd = Command::new(name);
        if let Some(name) = self.name {
            rootcmd = rootcmd.name(name);
        }
        if let Some(describe) = self.describe {
            rootcmd = rootcmd.about(describe);
        }
        if let Some(version) = self.version {
            rootcmd = rootcmd.version(version);
        }
        if let Some(author) = self.author {
            rootcmd = rootcmd.author(author);
        }
        for arg_data in &self.args {
            rootcmd = rootcmd.arg(arg_data.build());
        }
        for (name, subcmd) in &self.subcmds {
            rootcmd = rootcmd.subcommand(subcmd.build(name));
        }
        rootcmd
    }
    fn retrive(&'a self, matches: &ArgMatches) -> String {
        let mut values = vec![];
        for arg_data in &self.args {
            if let Some(value) = arg_data.retrive(&matches) {
                values.push(value);
            }
        }
        for (name, subcmd) in &self.subcmds {
            if let Some((subcmd_name, subcmd_matches)) = matches.subcommand() {
                if *name == subcmd_name {
                    values.push(subcmd.retrive(&subcmd_matches));
                    if self.subcmds.is_empty() {
                        values.push(name.to_string());
                    }
                }
            }
        }
        values.join("")
    }
}

#[derive(Debug)]
struct WrapArgData<'a> {
    data: &'a ArgData<'a>,
    value_name: String,
    pos_index: usize,
}

impl<'a> Deref for WrapArgData<'a> {
    type Target = ArgData<'a>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a> WrapArgData<'a> {
    fn new(data: &'a ArgData<'a>, pos_index: usize) -> Self {
        Self {
            data,
            value_name: data.name.to_case(Case::Cobol),
            pos_index,
        }
    }
    fn build(&'a self) -> Arg<'a> {
        let mut arg = Arg::new(self.name).required(self.required);
        if let Some(summary) = self.summary {
            let title = summary.trim();
            if title.len() > 0 {
                arg = arg.help(title);
            }
        }
        if self.positional {
            arg = arg.index(self.pos_index + 1).multiple_values(self.multiple);
        } else {
            arg = arg.long(self.name);
            if self.multiple {
                arg = arg
                    .multiple_values(true)
                    .use_value_delimiter(true)
                    .multiple_occurrences(true);
            }
            if !self.flag {
                arg = arg.value_name(&self.value_name);
            }
            if let Some(short) = self.short {
                arg = arg.short(short);
            }
            if let Some(choices) = &self.choices {
                if choices.len() > 1 {
                    arg = arg.possible_values(choices);
                }
            }
            if let Some(default) = self.default {
                arg = arg.default_value(default);
            }
        }
        arg
    }
    fn retrive(&self, matches: &ArgMatches) -> Option<String> {
        let name = self.name.to_case(Case::Snake);
        if !matches.is_present(self.name) {
            return None;
        }
        if self.flag {
            return Some(format!("selfc_{}=1\n", name));
        }
        if self.multiple {
            return matches.values_of(self.name).map(|values| {
                let values: Vec<String> = values.map(normalize_value).collect();
                format!("selfc_{}=( {} )\n", name, values.join(" "))
            });
        }
        matches
            .value_of(self.name)
            .map(|value| format!("selfc_{}={}\n", name, normalize_value(value)))
    }
}

fn normalize_value(value: &str) -> String {
    format!("\"{}\"", value.escape_debug())
}
