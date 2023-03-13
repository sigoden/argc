use crate::argc_value::ArgcValue;
use crate::param::{Param, ParamNames, PositionalParam, EXTRA_ARGS};
use crate::parser::{parse, Event, EventData, Position};
use crate::Result;
use anyhow::{anyhow, bail, Error};
use clap::{ArgMatches, Command};
use either::Either;
use std::collections::{HashMap, HashSet};

const ENTRYPOINT: &str = "main";

pub fn eval(source: &str, args: &[&str]) -> Result<Either<String, clap::Error>> {
    let events = parse(source)?;
    let cmd = Cli::new_from_events(&events)?;
    match cmd.eval(args)? {
        Either::Left(values) => Ok(Either::Left(ArgcValue::to_shell(values))),
        Either::Right(error) => Ok(Either::Right(error)),
    }
}

#[derive(Default)]
pub struct Cli {
    pub name: Option<String>,
    pub describe: Option<String>,
    pub positional_index: usize,
    pub params: Vec<(Box<dyn Param>, usize)>,
    pub subcommands: Vec<Cli>,
    // for conflict detecting
    pub names: ParamNames,
    // root only props
    pub root: Option<CliRoot>,
    pub aliases: Vec<String>,
}

#[derive(Default)]
pub struct CliRoot {
    pub author: Option<String>,
    pub version: Option<String>,
    pub fn_names: HashMap<String, Position>,
    pub help: Option<String>,
    pub main: bool,
    pub param_fns: Vec<(String, usize, bool)>,
}

impl Cli {
    pub fn new_from_events(events: &[Event]) -> Result<Self> {
        let mut root_cmd = Cli::default();
        let mut root_data = CliRoot::default();
        let mut is_root_scope = true;
        let mut maybe_subcommand: Option<Cli> = None;
        for event in events {
            let Event { data, position } = event.clone();
            match data {
                EventData::Describe(value) => {
                    if is_root_scope {
                        root_cmd.describe = Some(value);
                    }
                }
                EventData::Version(value) => {
                    if is_root_scope {
                        root_data.version = Some(value);
                    }
                }
                EventData::Author(value) => {
                    if is_root_scope {
                        root_data.author = Some(value);
                    }
                }
                EventData::Help(value) => {
                    if is_root_scope {
                        root_data.help = Some(value);
                    }
                }
                EventData::Cmd(value) => {
                    is_root_scope = false;
                    let mut cmd = Cli::default();
                    if !value.is_empty() {
                        cmd.describe = Some(value);
                    }
                    maybe_subcommand = Some(cmd);
                }
                EventData::Aliases(values) => {
                    if let Some(cmd) = &mut maybe_subcommand {
                        cmd.aliases = values.to_vec();
                        for name in values {
                            if let Some(pos) = root_data.fn_names.get(&name) {
                                bail!(
                                    "@alias(line {}) is conflicted with cmd or alias at line {}",
                                    position,
                                    pos
                                );
                            }
                            root_data.fn_names.insert(name.clone(), position);
                        }
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
                    let cmd = cmd.ok_or_else(|| unexpected_param(param.tag_name(), position))?;
                    root_data.collect_fns(position, &param.default_fn, &param.choices_fn);
                    cmd.add_param(param, position)?;
                }
                EventData::Positional(param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    let cmd = cmd.ok_or_else(|| unexpected_param(param.tag_name(), position))?;
                    root_data.collect_fns(position, &param.default_fn, &param.choices_fn);
                    cmd.add_param(param, position)?;
                }
                EventData::Flag(param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    let cmd = cmd.ok_or_else(|| unexpected_param(param.tag_name(), position))?;
                    cmd.add_param(param, position)?;
                }
                EventData::Func(name) => {
                    is_root_scope = false;

                    if let Some(pos) = root_data.fn_names.get(&name) {
                        bail!(
                            "{}(line {}) is conflicted with cmd or alias at line {}",
                            name,
                            position,
                            pos
                        )
                    }
                    root_data.fn_names.insert(name.clone(), position);
                    if let Some(mut cmd) = maybe_subcommand.take() {
                        cmd.name = Some(name.to_string());
                        cmd.extra_args()?;
                        root_cmd.subcommands.push(cmd);
                    } else if name == ENTRYPOINT {
                        root_data.main = true;
                    }
                }
                EventData::Unknown(name) => {
                    bail!("@{}(line {}) is unknown", name, position);
                }
            }
        }
        root_data.check_param_fns()?;
        root_cmd.extra_args()?;
        root_cmd.root = Some(root_data);
        Ok(root_cmd)
    }

    pub fn build_command(&self, name: &str) -> Result<Command> {
        let mut cmd = Command::new(name.to_string()).infer_long_args(true);
        if let Some(describe) = self.describe.as_ref() {
            cmd = cmd.about(describe);
        }
        if let Some(root_data) = &self.root {
            if let Some(version) = root_data.version.as_ref() {
                cmd = cmd.version(version);
            }
            if let Some(author) = root_data.author.as_ref() {
                cmd = cmd.author(author);
            }
            if !self.subcommands.is_empty() {
                cmd = cmd.infer_subcommands(true);
                if !root_data.main {
                    cmd = cmd.subcommand_required(true).arg_required_else_help(true);
                }
            }
            if let Some(help) = root_data.help.as_ref() {
                cmd = cmd
                    .disable_help_subcommand(true)
                    .subcommand(Command::new("help").about(help))
            } else {
                cmd = cmd.disable_help_subcommand(true);
            }
            for name in root_data.param_cmd_fns() {
                cmd = cmd.subcommand(Command::new(name).hide(true));
            }
        }
        if !self.aliases.is_empty() {
            cmd = cmd.visible_aliases(&self.aliases);
        }
        for (param, index) in &self.params {
            cmd = cmd.arg(param.build_arg(*index)?);
        }
        for subcommand in &self.subcommands {
            let subcommand = subcommand.build_command(subcommand.name.as_ref().unwrap())?;
            cmd = cmd.subcommand(subcommand);
        }
        cmd = cmd.help_template(self.help_template());
        Ok(cmd)
    }

    pub fn eval(&self, args: &[&str]) -> Result<Either<Vec<ArgcValue>, clap::Error>> {
        let name = args[0];
        let command = self.build_command(name)?;
        let res = command.try_get_matches_from(args);
        match res {
            Ok(matches) => {
                let values = self.get_args(&matches);
                Ok(Either::Left(values))
            }
            Err(err) => Ok(Either::Right(err)),
        }
    }

    pub fn get_args(&self, matches: &ArgMatches) -> Vec<ArgcValue> {
        let mut values = vec![];
        for (param, _) in &self.params {
            if let Some(value) = param.get_arg_value(matches) {
                values.push(value);
            }
        }
        let mut call_fn: Option<String> = None;
        if let Some(root_data) = self.root.as_ref() {
            for fn_name in root_data.param_cmd_fns() {
                if let Some((match_name, _)) = matches.subcommand() {
                    if fn_name.as_str() == match_name {
                        call_fn = Some(fn_name);
                    }
                }
            }
        }
        for subcommand in &self.subcommands {
            if let Some(fn_name) = &subcommand.name {
                if let Some((match_name, subcommand_matches)) = matches.subcommand() {
                    if *fn_name == match_name {
                        let subcommand_values = subcommand.get_args(subcommand_matches);
                        values.extend(subcommand_values);
                        call_fn = Some(fn_name.to_string());
                    }
                }
            }
        }
        call_fn = call_fn.or_else(|| {
            self.root.as_ref().and_then(|v| {
                if v.main {
                    Some("main".to_string())
                } else {
                    None
                }
            })
        });
        if let Some(fn_name) = call_fn {
            values.push(ArgcValue::FnName(fn_name));
        }
        values
    }

    fn add_param<T: Param + 'static>(&mut self, param: T, pos: Position) -> Result<()> {
        param.detect_conflict(&mut self.names, pos)?;
        let index = self.positional_index;
        if param.is_positional() {
            self.positional_index += 1;
        }
        self.params.push((Box::new(param), index));
        Ok(())
    }

    fn extra_args(&mut self) -> Result<()> {
        if self.positional_index == 0 {
            self.add_param(PositionalParam::extra(), 0)?;
        }
        Ok(())
    }

    fn help_template(&self) -> String {
        let mut lines = vec![];
        if let Some(root) = self.root.as_ref() {
            if root.version.is_some() {
                lines.push("{bin} {version}");
            }
            if root.author.is_some() {
                lines.push("{author}");
            }
        }
        if self.describe.is_some() {
            lines.push("{about}");
            lines.push("");
        }
        lines.push("USAGE: {usage}");
        lines.push("");
        let has_subcommands = !self.subcommands.is_empty();
        let has_arguments = self
            .params
            .iter()
            .any(|(p, _)| p.is_positional() && p.name() != EXTRA_ARGS);
        let has_options = self.params.iter().any(|(p, _)| !p.is_positional());
        if has_arguments {
            lines.push("ARGS:");
            lines.push("{positionals}");
            lines.push("");
        }

        if has_options {
            lines.push("OPTIONS:");
            lines.push("{options}");
            lines.push("");
        }

        if has_subcommands {
            lines.push("COMMANDS:");
            lines.push("{subcommands}");
            lines.push("");
        }
        lines.join("\n")
    }
}

impl CliRoot {
    pub fn collect_fns(
        &mut self,
        position: usize,
        default_fn: &Option<String>,
        choices_fn: &Option<String>,
    ) {
        if let Some(default_fn) = default_fn.as_ref() {
            self.param_fns
                .push((default_fn.to_string(), position, false));
        }
        if let Some(choices_fn) = choices_fn.as_ref() {
            self.param_fns
                .push((choices_fn.to_string(), position, true));
        }
    }

    fn check_param_fns(&self) -> Result<()> {
        for (name, pos, _) in &self.param_fns {
            if !self.fn_names.contains_key(name) {
                bail!("{}(line {}) is missing", name, pos,)
            }
        }
        Ok(())
    }

    fn param_cmd_fns(&self) -> HashSet<String> {
        self.param_fns
            .iter()
            .filter(|(_, _, is_choices_fn)| *is_choices_fn)
            .map(|(name, _, _)| name.to_string())
            .collect()
    }
}

fn unexpected_param(tag_name: &str, pos: Position) -> Error {
    anyhow!("{}(line {}) is unexpected, maybe miss @cmd?", tag_name, pos,)
}
