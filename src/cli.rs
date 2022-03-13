use crate::param::{Param, ParamNames, PositionalParam};
use crate::parser::{parse, Event, EventData, Position};
use crate::utils::*;
use crate::Result;
use anyhow::{anyhow, bail, Error};
use clap::{ArgMatches, Command};
use std::collections::HashMap;

const VARIABLE_PREFIX: &str = env!("CARGO_CRATE_NAME");

const ENTRYPOINT: &str = "main";

pub struct Runner<'a> {
    source: &'a str,
}

impl<'a> Runner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn run(&self, args: &[&'a str]) -> Result<std::result::Result<String, String>> {
        let events = parse(self.source)?;
        let default_shell_positional = PositionalParam::default_shell_positional();
        let cmd = Cmd::create(&events, &default_shell_positional)?;
        let name = args[0];
        let command = cmd.build(name)?;
        let res = command.try_get_matches_from(args);
        match res {
            Ok(matches) => {
                let values = cmd.retrieve(&matches, self);
                let output = to_string_retrieve_values(values);
                Ok(Ok(output))
            }
            Err(err) => Ok(Err(err.to_string())),
        }
    }
}

/// Run script with arguments, returns (stdout, stderr)
pub fn run<'a>(source: &'a str, args: &[&'a str]) -> Result<std::result::Result<String, String>> {
    let runner = Runner::new(source);
    runner.run(args)
}

#[derive(Default)]
struct Cmd<'a> {
    name: Option<(&'a str, String)>,
    describe: Option<&'a str>,
    positional_index: usize,
    params: Vec<(&'a dyn Param<'a>, usize)>,
    subcommands: Vec<Cmd<'a>>,
    // for conflict detecting
    names: ParamNames<'a>,
    // root only props
    root: Option<RootData<'a>>,
    aliases: Vec<&'a str>,
}

#[derive(Default)]
struct RootData<'a> {
    author: Option<&'a str>,
    version: Option<&'a str>,
    names: HashMap<&'a str, Position>,
    main: bool,
}

impl<'a> Cmd<'a> {
    fn create(
        events: &'a [Event<'a>],
        default_positional_param: &'a PositionalParam<'a>,
    ) -> Result<Self> {
        let mut root_cmd = Cmd::default();
        let mut root_data = RootData::default();
        let mut is_root_scope = true;
        let mut maybe_subcommand: Option<Cmd> = None;
        for Event { data, position } in events {
            match data {
                EventData::Describe(value) => {
                    if is_root_scope {
                        root_cmd.describe = Some(*value);
                    }
                }
                EventData::Version(value) => {
                    if is_root_scope {
                        root_data.version = Some(*value);
                    }
                }
                EventData::Author(value) => {
                    if is_root_scope {
                        root_data.author = Some(*value);
                    }
                }
                EventData::Cmd(value) => {
                    is_root_scope = false;
                    let mut cmd = Cmd::default();
                    if !value.is_empty() {
                        cmd.describe = Some(*value);
                    }
                    maybe_subcommand = Some(cmd);
                }
                EventData::Aliases(values) => {
                    if let Some(cmd) = &mut maybe_subcommand {
                        for name in values {
                            if let Some(pos) = root_data.names.get(name) {
                                bail!(
                                    "@alias(line {}) is conflicted with cmd or alias at line {}",
                                    position,
                                    pos
                                );
                            }
                            root_data.names.insert(name, *position);
                        }
                        cmd.aliases = values.to_vec();
                    } else {
                        bail!("@alias(line {}) is unexpected", position)
                    }
                }
                EventData::Option(param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    let cmd = cmd.ok_or_else(|| unexpected_param(param.tag_name(), *position))?;
                    cmd.add_param(param, *position)?;
                }
                EventData::Positional(param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    let cmd = cmd.ok_or_else(|| unexpected_param(param.tag_name(), *position))?;
                    cmd.add_param(param, *position)?;
                }
                EventData::Flag(param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    let cmd = cmd.ok_or_else(|| unexpected_param(param.tag_name(), *position))?;
                    cmd.add_param(param, *position)?;
                }
                EventData::Func(name) => {
                    is_root_scope = false;
                    if let Some(mut cmd) = maybe_subcommand.take() {
                        if let Some(pos) = root_data.names.get(name) {
                            bail!(
                                "{}(line {}) is conflicted with cmd or alias at line {}",
                                name,
                                position,
                                pos
                            )
                        }
                        root_data.names.insert(name, *position);
                        cmd.name = Some((name, to_kebab_case(name)));
                        cmd.maybe_add_default_positional(default_positional_param)?;
                        root_cmd.subcommands.push(cmd);
                    } else if *name == ENTRYPOINT {
                        root_data.main = true;
                    }
                }
                EventData::Unknown(name) => {
                    bail!("@{}(line {}) is unknown", name, position);
                }
            }
        }
        root_cmd.maybe_add_default_positional(default_positional_param)?;
        root_cmd.root = Some(root_data);
        Ok(root_cmd)
    }

    fn build(&'a self, name: &'a str) -> Result<Command<'a>> {
        let mut cmd = Command::new(name);
        if let Some(describe) = self.describe {
            cmd = cmd.about(describe);
        }
        if let Some(root_data) = &self.root {
            if let Some(version) = root_data.version {
                cmd = cmd.version(version);
            }
            if let Some(author) = root_data.author {
                cmd = cmd.author(author);
            }
            if !self.subcommands.is_empty() && !root_data.main {
                cmd = cmd.subcommand_required(true).arg_required_else_help(true);
            }
        }
        if !self.aliases.is_empty() {
            cmd = cmd.visible_aliases(&self.aliases);
        }
        for (param, index) in &self.params {
            cmd = cmd.arg(param.build_arg(*index)?);
        }
        for subcommand in &self.subcommands {
            let subcommand = subcommand.build(subcommand.name.as_ref().unwrap().1.as_str())?;
            cmd = cmd.subcommand(subcommand);
        }
        Ok(cmd)
    }

    fn retrieve(&'a self, matches: &ArgMatches, runner: &Runner) -> Vec<RetrieveValue> {
        let mut values = vec![];
        for (param, _) in &self.params {
            if let Some(value) = param.retrieve_value(matches) {
                values.push(value);
            }
        }
        let mut call_fn: Option<&str> = None;
        for subcommand in &self.subcommands {
            if let Some((fn_name, cmd_name)) = &subcommand.name {
                if let Some((match_name, subcommand_matches)) = matches.subcommand() {
                    if cmd_name.as_str() == match_name {
                        let subcommand_values = subcommand.retrieve(subcommand_matches, runner);
                        values.extend(subcommand_values);
                        call_fn = Some(fn_name);
                    }
                }
            }
        }
        call_fn = call_fn.or_else(|| {
            self.root
                .as_ref()
                .and_then(|v| if v.main { Some("main") } else { None })
        });
        if let Some(fn_name) = call_fn {
            values.push(RetrieveValue::FnName(fn_name));
        }
        values
    }

    fn add_param<T: 'a + Param<'a>>(&mut self, param: &'a T, pos: Position) -> Result<()> {
        param.detect_conflict(&mut self.names, pos)?;
        let index = self.positional_index;
        if param.is_positional() {
            self.positional_index += 1;
        }
        self.params.push((param, index));
        Ok(())
    }

    fn maybe_add_default_positional(
        &mut self,
        default_positional_param: &'a PositionalParam<'a>,
    ) -> Result<()> {
        if self.positional_index == 0 {
            self.add_param(default_positional_param, 0)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum RetrieveValue<'a> {
    Single(&'a str, String),
    Multiple(&'a str, Vec<String>),
    PositionalSingle(&'a str, String),
    PositionalMultiple(&'a str, Vec<String>),
    FnName(&'a str),
}

fn to_string_retrieve_values(values: Vec<RetrieveValue>) -> String {
    let mut variables = vec![];
    let mut positional_args = vec![];
    for value in values {
        match value {
            RetrieveValue::Single(name, value) => {
                variables.push(format!("{}_{}={}", VARIABLE_PREFIX, name, value));
            }
            RetrieveValue::Multiple(name, values) => {
                variables.push(format!(
                    "{}_{}=( {} )",
                    VARIABLE_PREFIX,
                    name,
                    values.join(" ")
                ));
            }
            RetrieveValue::PositionalSingle(name, value) => {
                variables.push(format!("{}_{}={}", VARIABLE_PREFIX, name, &value));
                positional_args.push(value);
            }
            RetrieveValue::PositionalMultiple(name, values) => {
                variables.push(format!(
                    "{}_{}=( {} )",
                    VARIABLE_PREFIX,
                    name,
                    values.join(" ")
                ));
                positional_args.extend(values);
            }
            RetrieveValue::FnName(name) => {
                if positional_args.is_empty() {
                    variables.push(name.to_string());
                } else {
                    variables.push(format!("{} {}", name, positional_args.join(" ")));
                }
            }
        }
    }
    variables.join("\n")
}

fn unexpected_param(tag_name: &str, pos: Position) -> Error {
    anyhow!("{}(line {}) is unexpected, maybe miss @cmd?", tag_name, pos,)
}
