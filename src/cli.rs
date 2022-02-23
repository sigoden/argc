use crate::parser::{parse, ArgData, ArgKind, Event, EventData, Position};
use crate::Result;
use anyhow::bail;
use clap::{Arg, ArgMatches, Command};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::ops::Deref;

const VARIABLE_PREFIX: &'static str = env!("CARGO_CRATE_NAME");

const ENTRYPOINT: &'static str = "main";

/// Run script with arguments, returns (stdout, stderr)
pub fn run<'a>(source: &'a str, args: &[&'a str]) -> Result<std::result::Result<String, String>> {
    let events = parse(source)?;
    let name = args[0];
    let mut rootcmd = Cmd::from_events(&events)?;
    rootcmd.name = Some((&name, name.to_string()));
    let command = rootcmd.build()?;
    let res = command.try_get_matches_from(args);
    match res {
        Ok(matches) => Ok(Ok(rootcmd.retrive(&matches))),
        Err(err) => Ok(Err(err.to_string())),
    }
}

#[derive(Debug, Default)]
struct Cmd<'a> {
    name: Option<(&'a str, String)>,
    describe: Option<&'a str>,
    postional_idx: usize,
    args: Vec<WrapArgData<'a>>,
    subcmds: HashMap<&'a str, Cmd<'a>>,
    // for conflict detecting
    names: (HashMap<&'a str, Position>, HashMap<char, Position>),
    // root only props
    root: Option<RootData<'a>>,
}

#[derive(Debug, Default)]
struct RootData<'a> {
    author: Option<&'a str>,
    version: Option<&'a str>,
    main: bool,
}

impl<'a> Cmd<'a> {
    fn from_events(events: &'a [Event]) -> Result<Self> {
        let mut rootcmd = Cmd::default();
        let mut rootdata = RootData::default();
        let mut is_root_scope = true;
        let mut maybe_subcmd: Option<Cmd> = None;
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
                        rootdata.version = Some(*value);
                    }
                }
                EventData::Author(value) => {
                    if let Some(_) = maybe_subcmd {
                    } else {
                        rootdata.author = Some(*value);
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
                        cmd.add_arg(arg_data, position)?;
                    } else {
                        if is_root_scope {
                            rootcmd.add_arg(arg_data, position)?;
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
                            rootdata.main = true;
                        }
                    }
                }
                EventData::Unknown(name) => {
                    bail!("@{}(line {}) is unknown", name, position);
                }
            }
        }
        rootcmd.root = Some(rootdata);
        Ok(rootcmd)
    }
    fn build(&'a self) -> Result<Command<'a>> {
        if self.name.is_none() {
            bail!("command must have name");
        }
        let (_, cmd_name) = self.name.as_ref().unwrap();
        let mut cmd = Command::new(cmd_name);
        if let Some(describe) = self.describe {
            cmd = cmd.about(describe);
        }
        if let Some(rootdata) = &self.root {
            if let Some(version) = rootdata.version {
                cmd = cmd.version(version);
            }
            if let Some(author) = rootdata.author {
                cmd = cmd.author(author);
            }
            if self.subcmds.len() > 0 && !rootdata.main {
                cmd = cmd.subcommand_required(true);
            }
        }
        for arg_data in &self.args {
            cmd = cmd.arg(arg_data.build()?);
        }
        for (_, subcmd) in &self.subcmds {
            let subcmd = subcmd.build()?;
            cmd = cmd.subcommand(subcmd);
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
        let mut call_fn: Option<String> = None;
        for (_, subcmd) in &self.subcmds {
            if let Some((fn_name, cmd_name)) = &subcmd.name {
                if let Some((match_name, subcmd_matches)) = matches.subcommand() {
                    if cmd_name.as_str() == match_name {
                        values.push(subcmd.retrive(&subcmd_matches));
                        call_fn = Some(fn_name.to_string());
                    }
                }
            }
        }
        call_fn = call_fn.or_else(|| {
            self.root
                .as_ref()
                .and_then(|v| if v.main { Some(format!("main")) } else { None })
        });
        if let Some(fn_name) = call_fn {
            values.push(format!("{}__call={}", VARIABLE_PREFIX, fn_name));
        }
        values.join("")
    }
    fn add_arg(&mut self, arg_data: &'a ArgData, position: &Position) -> Result<()> {
        let arg_data = WrapArgData::new(arg_data, self.postional_idx);
        arg_data.detect_conflict(&mut self.names, *position)?;
        if arg_data.is_positional() {
            self.postional_idx += 1;
        }
        self.args.push(arg_data);
        Ok(())
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
            return Some(format!("{}_{}=1\n", VARIABLE_PREFIX, name));
        }
        if self.multiple {
            return matches.values_of(self.name).map(|values| {
                let values: Vec<String> = values.map(normalize_value).collect();
                format!("{}_{}=( {} )\n", VARIABLE_PREFIX, name, values.join(" "))
            });
        }
        matches
            .value_of(self.name)
            .map(|value| format!("{}_{}={}\n", VARIABLE_PREFIX, name, normalize_value(value)))
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
