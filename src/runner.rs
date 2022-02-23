use crate::parser::{parse, ArgData, ArgKind, Event, EventData, Position};
use crate::Result;
use anyhow::bail;
use clap::{Arg, ArgMatches, Command};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::ops::Deref;

const ENTRYPOINT: &'static str = "main";

/// Run script with arguments, returns (stdout, stderr)
pub fn run<'a>(source: &'a str, args: &[&'a str]) -> Result<(Option<String>, Option<String>)> {
    let events = parse(source)?;
    let name = args[0];
    let mut rootcmd = Cmd::create(&events)?;
    rootcmd.name = Some((&name, name.to_string()));
    let command = rootcmd.build()?;
    let res = command.try_get_matches_from(args);
    match res {
        Ok(matches) => {
            let mut output = rootcmd.retrive(&matches);
            if matches.subcommand_name().is_none() && rootcmd.has_main_fn == true {
                output.push_str(ENTRYPOINT)
            }
            Ok((Some(output), None))
        }
        Err(err) => Ok((Some(format!("exit 1")), Some(err.to_string()))),
    }
}

#[derive(Debug, Default)]
struct Cmd<'a> {
    name: Option<(&'a str, String)>,
    author: Option<&'a str>,
    describe: Option<&'a str>,
    version: Option<&'a str>,
    postional_idx: usize,
    args: Vec<WrapArgData<'a>>,
    subcmds: HashMap<&'a str, Cmd<'a>>,
    has_main_fn: bool,
    // for conflict detecting
    names: (HashMap<&'a str, Position>, HashMap<char, Position>),
}

impl<'a> Cmd<'a> {
    fn create(events: &'a [Event]) -> Result<Self> {
        let mut maybe_subcmd: Option<Cmd> = None;
        let mut rootcmd = Cmd::default();
        let mut is_root_scope = true;
        for Event { data, position } in events {
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
                    is_root_scope = false;
                    let mut cmd = Cmd::default();
                    if value.len() > 0 {
                        cmd.describe = Some(*value);
                    }
                    maybe_subcmd = Some(cmd);
                }
                EventData::Arg(arg_data) => {
                    if let Some(cmd) = &mut maybe_subcmd {
                        let arg_data = WrapArgData::new(arg_data, cmd.postional_idx);
                        arg_data.detect_conflict(&mut cmd.names, *position)?;
                        if arg_data.is_positional() {
                            cmd.postional_idx += 1;
                        }
                        cmd.args.push(arg_data);
                    } else {
                        if is_root_scope {
                            let arg_data = WrapArgData::new(arg_data, rootcmd.postional_idx);
                            arg_data.detect_conflict(&mut rootcmd.names, *position)?;
                            if arg_data.is_positional() {
                                rootcmd.postional_idx += 1;
                            }
                            rootcmd.args.push(arg_data);
                        } else {
                            bail!(
                                "{}(line {}) is unexpected, maybe miss @cmd?",
                                arg_data.kind,
                                position
                            )
                        }
                    }
                }
                EventData::Func(name) => {
                    is_root_scope = false;
                    if let Some(mut cmd) = maybe_subcmd.take() {
                        if rootcmd.subcmds.get(name).is_some() {
                            bail!("{}(line {}) already exists", name, position)
                        }
                        cmd.name = Some((name, name.to_case(Case::Kebab)));
                        rootcmd.subcmds.insert(*name, cmd);
                    } else {
                        if *name == ENTRYPOINT {
                            rootcmd.has_main_fn = true;
                        }
                    }
                    // println!("{:?}", rootcmd.subcmds)
                }
                EventData::Unexpect(name) => {
                    bail!("@{}(line {}) is invalid", name, position);
                }
            }
        }
        Ok(rootcmd)
    }
    fn build(&'a self) -> Result<Command<'a>> {
        if self.name.is_none() {
            bail!("Why miss command name");
        }
        let (_, cmd_name) = self.name.as_ref().unwrap();
        let mut cmd = Command::new(cmd_name);
        if let Some(describe) = self.describe {
            cmd = cmd.about(describe);
        }
        if let Some(version) = self.version {
            cmd = cmd.version(version);
        }
        if let Some(author) = self.author {
            cmd = cmd.author(author);
        }
        for arg_data in &self.args {
            cmd = cmd.arg(arg_data.build()?);
        }
        for (_, subcmd) in &self.subcmds {
            let subcmd = subcmd.build()?;
            cmd = cmd.subcommand(subcmd);
        }
        if self.subcmds.len() > 0 && !self.has_main_fn {
            cmd = cmd.subcommand_required(true);
        }
        Ok(cmd)
    }
    fn retrive(&'a self, matches: &ArgMatches) -> String {
        let mut values = vec![];
        for arg_data in &self.args {
            if let Some(value) = arg_data.retrive_match_value(&matches) {
                values.push(value);
            }
        }
        for (_, subcmd) in &self.subcmds {
            if let Some((fn_name, cmd_name)) = &subcmd.name {
                if let Some((match_name, subcmd_matches)) = matches.subcommand() {
                    if cmd_name.as_str() == match_name {
                        values.push(subcmd.retrive(&subcmd_matches));
                        values.push(fn_name.to_string());
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
        let value_name = data
            .value_name
            .map(|v| v.to_owned())
            .unwrap_or_else(|| data.name.to_case(Case::Cobol));
        Self {
            data,
            value_name,
            pos_index,
        }
    }
    fn build(&'a self) -> Result<Arg<'a>> {
        let mut arg = Arg::new(self.name);
        if let Some(summary) = self.summary {
            let title = summary.trim();
            if title.len() > 0 {
                arg = arg.help(title);
            }
        }
        let arg = match &self.kind {
            ArgKind::Flag => {
                let mut arg = arg.long(self.name);
                if let Some(s) = self.short {
                    arg = arg.short(s);
                }
                arg
            }
            ArgKind::Option => {
                let mut arg = arg
                    .long(self.name)
                    .required(self.required)
                    .value_name(&self.value_name);
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
                arg
            }
            ArgKind::Positional => {
                let mut arg = arg
                    .index(self.pos_index + 1)
                    .required(self.required)
                    .value_name(&self.value_name);

                if self.multiple {
                    arg = arg.multiple_values(true)
                }
                arg
            }
        };
        Ok(arg)
    }
    fn retrive_match_value(&self, matches: &ArgMatches) -> Option<String> {
        let name = self.name.to_case(Case::Snake);
        if !matches.is_present(self.name) {
            return None;
        }
        if self.kind == ArgKind::Flag {
            return Some(format!("argc_{}=1\n", name));
        }
        if self.multiple {
            return matches.values_of(self.name).map(|values| {
                let values: Vec<String> = values.map(normalize_value).collect();
                format!("argc_{}=( {} )\n", name, values.join(" "))
            });
        }
        matches
            .value_of(self.name)
            .map(|value| format!("argc_{}={}\n", name, normalize_value(value)))
    }
    fn detect_conflict(
        &self,
        names: &mut (HashMap<&'a str, Position>, HashMap<char, Position>),
        current: Position,
    ) -> Result<()> {
        match self.kind {
            ArgKind::Positional => {
                if let Some(position) = names.0.get(self.name) {
                    bail!(
                        "{}(line {}) has `{}` already exists at line {}",
                        self.kind,
                        current,
                        self.name,
                        position,
                    );
                } else {
                    names.0.insert(self.name, current);
                }
            }
            _ => {
                if let Some(position) = names.0.get(self.name) {
                    bail!(
                        "{}(line {}) has --{} already exists at line {}",
                        self.kind,
                        current,
                        self.name,
                        position,
                    )
                } else {
                    names.0.insert(self.name, current);
                }
                if let Some(short) = self.short {
                    if let Some(position) = names.1.get(&short) {
                        bail!(
                            "{}(line {}) has -{} already exists at line {}",
                            self.kind,
                            current,
                            short,
                            position,
                        )
                    } else {
                        names.1.insert(short, current);
                    }
                }
            }
        }
        Ok(())
    }
}

fn normalize_value(value: &str) -> String {
    format!("\"{}\"", value.escape_debug())
}
