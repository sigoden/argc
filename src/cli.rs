use crate::param::{Param, ParamNames, PositionalParam};
use crate::parser::{parse, Event, EventData, Position};
use crate::utils::hyphens_to_underscores;
use crate::Result;
use anyhow::{anyhow, bail, Error};
use clap::{ArgMatches, Command};
use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, HashSet};

const VARIABLE_PREFIX: &str = env!("CARGO_CRATE_NAME");

const ENTRYPOINT: &str = "main";

pub struct Cli<'a> {
    source: &'a str,
}

impl<'a> Cli<'a> {
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

    pub fn compgen(&self, args: &[&'a str]) -> Result<Vec<String>> {
        let events = parse(self.source)?;
        let cmd_comp = CmdComp::create(&events);
        let mut i = 1;
        let len = args.len();
        let mut omitted: HashSet<String> = HashSet::default();
        let mut cmd_comp = &cmd_comp;
        let mut positional_index = 0;
        let mut unknown_arg = false;
        while i < len {
            let arg = args[i];
            if let Some(name) = cmd_comp.mappings.get(arg) {
                if arg.starts_with('-') {
                    if let Some((short, choices, multiple)) = cmd_comp.options.get(name) {
                        if i == len - 1 {
                            return Ok(choices.clone());
                        }
                        if *multiple {
                            while i + 1 < len && !args[i + 1].starts_with('-') {
                                i += 1;
                            }
                        } else {
                            if !args[i + 1].starts_with('-') {
                                i += 1;
                            }
                            omitted.insert(name.to_string());
                            if let Some(short) = short {
                                omitted.insert(short.to_string());
                            }
                        }
                    } else if let Some(short) = cmd_comp.flags.get(name) {
                        omitted.insert(name.to_string());
                        if let Some(short) = short {
                            omitted.insert(short.to_string());
                        }
                    }
                } else if let Some(cmd) = cmd_comp.subcommands.get(name) {
                    cmd_comp = cmd;
                    omitted.clear();
                    positional_index = 0;
                }
            } else if arg.starts_with('-') {
                unknown_arg = true;
                positional_index = 0;
            } else if !unknown_arg {
                positional_index += 1;
            }
            i += 1;
        }
        let mut output = vec![];
        for name in cmd_comp.mappings.keys() {
            if !omitted.contains(name) {
                output.push(name.to_string());
            }
        }
        if positional_index >= cmd_comp.positionals.len() {
            if let Some((name, _)) = cmd_comp.positionals.last() {
                if name.ends_with("...") {
                    output.push(name.to_string());
                }
            }
        } else if let Some((name, choices)) = cmd_comp.positionals.iter().nth(positional_index) {
            if choices.is_empty() {
                output.push(name.to_string())
            } else {
                output.extend(choices.to_vec());
            }
        }
        Ok(output)
    }
}

#[derive(Default)]
struct Cmd<'a> {
    name: Option<&'a str>,
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
    help: Option<&'a str>,
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
                EventData::Help(value) => {
                    if is_root_scope {
                        root_data.help = Some(*value);
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
                        cmd.name = Some(name);
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
        let mut cmd = Command::new(name).infer_long_args(true);
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
            if !self.subcommands.is_empty() {
                cmd = cmd.infer_subcommands(true);
                if !root_data.main {
                    cmd = cmd.subcommand_required(true).arg_required_else_help(true);
                }
            }
            if let Some(help) = root_data.help {
                cmd = cmd.subcommand(Command::new("help").about(help))
            } else {
                cmd = cmd.disable_help_subcommand(true);
            }
        }
        if !self.aliases.is_empty() {
            cmd = cmd.visible_aliases(&self.aliases);
        }
        for (param, index) in &self.params {
            cmd = cmd.arg(param.build_arg(*index)?);
        }
        for subcommand in &self.subcommands {
            let subcommand = subcommand.build(subcommand.name.as_ref().unwrap())?;
            cmd = cmd.subcommand(subcommand);
        }
        Ok(cmd)
    }

    fn retrieve(&'a self, matches: &ArgMatches, cli: &Cli) -> Vec<RetrieveValue> {
        let mut values = vec![];
        for (param, _) in &self.params {
            if let Some(value) = param.retrieve_value(matches) {
                values.push(value);
            }
        }
        let mut call_fn: Option<&str> = None;
        for subcommand in &self.subcommands {
            if let Some(fn_name) = &subcommand.name {
                if let Some((match_name, subcommand_matches)) = matches.subcommand() {
                    if *fn_name == match_name {
                        let subcommand_values = subcommand.retrieve(subcommand_matches, cli);
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

#[derive(Debug, PartialEq, Eq)]
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
                variables.push(format!(
                    "{}_{}={}",
                    VARIABLE_PREFIX,
                    hyphens_to_underscores(name),
                    value
                ));
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
                variables.push(format!(
                    "{}_{}={}",
                    VARIABLE_PREFIX,
                    hyphens_to_underscores(name),
                    &value
                ));
                positional_args.push(value);
            }
            RetrieveValue::PositionalMultiple(name, values) => {
                variables.push(format!(
                    "{}_{}=( {} )",
                    VARIABLE_PREFIX,
                    hyphens_to_underscores(name),
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

#[derive(Debug, Default)]
pub struct CmdComp {
    aliases: IndexSet<String>,
    mappings: IndexMap<String, String>,
    options: HashMap<String, (Option<String>, Vec<String>, bool)>,
    flags: HashMap<String, Option<String>>,
    positionals: IndexMap<String, Vec<String>>,
    subcommands: IndexMap<String, CmdComp>,
}

impl CmdComp {
    fn create(events: &[Event]) -> Self {
        let mut root_cmd = CmdComp::default();
        let mut maybe_subcommand: Option<CmdComp> = None;
        let mut is_root_scope = true;
        let mut help_subcommand = false;
        for Event { data, .. } in events {
            match data {
                EventData::Help(_) => {
                    help_subcommand = true;
                }
                EventData::Cmd(_) => {
                    is_root_scope = false;
                    maybe_subcommand = Some(CmdComp::default())
                }
                EventData::Aliases(aliases) => {
                    if let Some(cmd) = &mut maybe_subcommand {
                        cmd.aliases.extend(aliases.iter().map(|v| v.to_string()))
                    }
                }
                EventData::Option(option_param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    if let Some(cmd) = cmd {
                        let name = format!("--{}", option_param.name);
                        let short = if let Some(short) = option_param.short.as_ref() {
                            let short = format!("-{}", short);
                            cmd.mappings.insert(short.clone(), name.clone());
                            Some(short)
                        } else {
                            None
                        };
                        let choices = match &option_param.choices {
                            Some(choices) => choices.iter().map(|v| v.to_string()).collect(),
                            None => vec![],
                        };
                        cmd.mappings.insert(name.clone(), name.clone());
                        cmd.options
                            .insert(name.clone(), (short, choices, option_param.multiple));
                    }
                }
                EventData::Flag(flag_param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    if let Some(cmd) = cmd {
                        let name = format!("--{}", flag_param.name);
                        let short = if let Some(short) = flag_param.short.as_ref() {
                            let short = format!("-{}", short);
                            cmd.mappings.insert(short.clone(), name.clone());
                            Some(short)
                        } else {
                            None
                        };
                        cmd.mappings.insert(name.clone(), name.clone());
                        cmd.flags.insert(name, short);
                    }
                }
                EventData::Positional(positional_param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    if let Some(cmd) = cmd {
                        let multiple = if positional_param.multiple { "..." } else { "" };
                        let choices = match &positional_param.choices {
                            Some(choices) => choices.iter().map(|v| v.to_string()).collect(),
                            None => vec![],
                        };
                        cmd.positionals.insert(
                            format!("<{}>{}", positional_param.name.to_uppercase(), multiple),
                            choices,
                        );
                    }
                }
                EventData::Func(name) => {
                    is_root_scope = false;
                    let name = name.to_string();
                    if let Some(mut cmd) = maybe_subcommand.take() {
                        root_cmd.mappings.insert(name.clone(), name.clone());
                        for alias in cmd.aliases.drain(..) {
                            root_cmd.mappings.insert(alias, name.clone());
                        }
                        root_cmd.subcommands.insert(name.clone(), cmd);
                    }
                }
                _ => {}
            }
        }
        if help_subcommand {
            let mut cmd = CmdComp::default();
            cmd.positionals.insert(
                "<CMD>".to_string(),
                root_cmd.subcommands.keys().map(|v| v.to_string()).collect(),
            );
            root_cmd
                .mappings
                .insert("help".to_string(), "help".to_string());
            root_cmd.subcommands.insert("help".into(), cmd);
        }
        root_cmd
    }
}
