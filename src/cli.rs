use crate::param::{Param, ParamNames, PositionalParam, EXTRA_ARGS};
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
        let cmd = Cmd::create(events)?;
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
struct Cmd {
    name: Option<String>,
    describe: Option<String>,
    positional_index: usize,
    params: Vec<(Box<dyn Param>, usize)>,
    subcommands: Vec<Cmd>,
    // for conflict detecting
    names: ParamNames,
    // root only props
    root: Option<RootData>,
    aliases: Vec<String>,
}

#[derive(Default)]
struct RootData {
    author: Option<String>,
    version: Option<String>,
    names: HashMap<String, Position>,
    help: Option<String>,
    main: bool,
}

impl Cmd {
    fn create(events: Vec<Event>) -> Result<Self> {
        let mut root_cmd = Cmd::default();
        let mut root_data = RootData::default();
        let mut is_root_scope = true;
        let mut maybe_subcommand: Option<Cmd> = None;
        for Event { data, position } in events {
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
                    let mut cmd = Cmd::default();
                    if !value.is_empty() {
                        cmd.describe = Some(value);
                    }
                    maybe_subcommand = Some(cmd);
                }
                EventData::Aliases(values) => {
                    if let Some(cmd) = &mut maybe_subcommand {
                        cmd.aliases = values.to_vec();
                        for name in values {
                            if let Some(pos) = root_data.names.get(&name) {
                                bail!(
                                    "@alias(line {}) is conflicted with cmd or alias at line {}",
                                    position,
                                    pos
                                );
                            }
                            root_data.names.insert(name.clone(), position);
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
                    cmd.add_param(param, position)?;
                }
                EventData::Positional(param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    let cmd = cmd.ok_or_else(|| unexpected_param(param.tag_name(), position))?;
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
                    if let Some(mut cmd) = maybe_subcommand.take() {
                        if let Some(pos) = root_data.names.get(&name) {
                            bail!(
                                "{}(line {}) is conflicted with cmd or alias at line {}",
                                name,
                                position,
                                pos
                            )
                        }
                        root_data.names.insert(name.clone(), position);
                        cmd.name = Some(name.to_string());
                        cmd.maybe_extra_positional_args()?;
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
        root_cmd.maybe_extra_positional_args()?;
        root_cmd.root = Some(root_data);
        Ok(root_cmd)
    }

    fn build(&self, name: &str) -> Result<Command> {
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
        cmd = cmd.help_template(self.help_template());
        Ok(cmd)
    }

    fn retrieve(&self, matches: &ArgMatches, cli: &Cli) -> Vec<RetrieveValue> {
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
            values.push(RetrieveValue::FnName(fn_name.to_string()));
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

    fn maybe_extra_positional_args(&mut self) -> Result<()> {
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
        } else {
            lines.push("{bin}");
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
        if has_arguments {
            lines.push("ARGS:");
            lines.push("{positionals}");
            lines.push("");
        }

        lines.push("OPTIONS:");
        lines.push("{options}");
        lines.push("");

        if has_subcommands {
            lines.push("COMMANDS:");
            lines.push("{subcommands}");
            lines.push("");
        }
        lines.join("\n")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RetrieveValue {
    Single(String, String),
    Multiple(String, Vec<String>),
    PositionalSingle(String, String),
    PositionalMultiple(String, Vec<String>),
    FnName(String),
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
                    hyphens_to_underscores(&name),
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
                    hyphens_to_underscores(&name),
                    &value
                ));
                positional_args.push(value);
            }
            RetrieveValue::PositionalMultiple(name, values) => {
                variables.push(format!(
                    "{}_{}=( {} )",
                    VARIABLE_PREFIX,
                    hyphens_to_underscores(&name),
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
