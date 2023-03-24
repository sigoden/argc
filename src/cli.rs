use crate::argc_value::ArgcValue;
use crate::completion::DYNAMIC_COMPGEN_FN;
use crate::param::{Param, ParamNames, PositionalParam, EXTRA_ARGS};
use crate::parser::{parse, Event, EventData, EventScope, Position};
use crate::utils::{argmap, escape_shell_words, split_shell_words};
use crate::Result;
use anyhow::bail;
use clap::{ArgMatches, Command};
use either::Either;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

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
    name: Option<String>,
    fn_name: Option<String>,
    describe: Option<String>,
    positional_index: usize,
    params: Vec<(Box<dyn Param>, usize)>,
    subcommands: Vec<Cli>,
    help: Option<String>,
    author: Option<String>,
    version: Option<String>,
    // for conflict detecting
    names: ParamNames,
    root: Arc<RefCell<RootData>>,
    aliases: Vec<String>,
}

impl Cli {
    pub fn new_from_events(events: &[Event]) -> Result<Self> {
        let mut root_cmd = Cli::default();
        let root_data = root_cmd.root.clone();
        for event in events {
            let Event { data, position } = event.clone();
            match data {
                EventData::Describe(value) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@describe", position)?;
                    cmd.describe = Some(value);
                }
                EventData::Version(value) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@version", position)?;
                    cmd.version = Some(value);
                }
                EventData::Author(value) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@author", position)?;
                    cmd.author = Some(value);
                }
                EventData::Help(value) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@help", position)?;
                    cmd.help = Some(value);
                }
                EventData::Cmd(value) => {
                    root_data.borrow_mut().scope = EventScope::CmdStart;
                    let mut subcmd = root_cmd.create_subcommand();
                    if !value.is_empty() {
                        subcmd.describe = Some(value.clone());
                    }
                }
                EventData::Aliases(values) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@help", position)?;
                    cmd.aliases = values.to_vec();
                    for name in values {
                        if let Some(pos) = root_data.borrow().fns.get(&name) {
                            bail!(
                                "@alias(line {}) is conflicted with cmd or alias at line {}",
                                position,
                                pos
                            );
                        }
                        root_data.borrow_mut().fns.insert(name.clone(), position);
                    }
                }
                EventData::Option(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    root_data.borrow_mut().add_default_choices_fn(
                        position,
                        &param.default_fn,
                        &param.choices_fn,
                    );
                    cmd.add_param(param, position)?;
                }
                EventData::Positional(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    root_data.borrow_mut().add_default_choices_fn(
                        position,
                        &param.default_fn,
                        &param.choices_fn,
                    );
                    cmd.add_param(param, position)?;
                }
                EventData::Flag(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    cmd.add_param(param, position)?;
                }
                EventData::Func(name) => {
                    if let Some(pos) = root_data.borrow_mut().fns.get(&name) {
                        bail!(
                            "{}(line {}) is conflicted with cmd or alias at line {}",
                            name,
                            position,
                            pos
                        )
                    }
                    root_data.borrow_mut().fns.insert(name.clone(), position);
                    if root_data.borrow().scope == EventScope::CmdStart {
                        let (parent, child) = match name.split_once("::") {
                            None => (name.as_str(), None),
                            Some((parent, child)) => {
                                if child.is_empty() {
                                    bail!("{}(line {}) is invalid", name, position);
                                }
                                (parent, Some(child))
                            }
                        };
                        match child {
                            None => {
                                let cmd = root_cmd.subcommands.last_mut().unwrap();
                                cmd.name = Some(parent.to_string());
                                cmd.fn_name = Some(name.to_string());
                                cmd.extra_args()?;
                            }
                            Some(child) => {
                                let mut cmd = root_cmd.subcommands.pop().unwrap();
                                cmd.name = Some(child.to_string());
                                cmd.fn_name = Some(name.to_string());
                                cmd.extra_args()?;
                                match root_cmd
                                    .subcommands
                                    .iter_mut()
                                    .find(|v| v.name == Some(parent.into()))
                                {
                                    Some(parent_cmd) => {
                                        parent_cmd.subcommands.push(cmd);
                                    }
                                    None => {
                                        bail!("{}(line {}) has no parent", name, position);
                                    }
                                }
                            }
                        }
                    } else if name == DYNAMIC_COMPGEN_FN {
                        root_data.borrow_mut().dynamic_compgen = true;
                    }
                    root_data.borrow_mut().scope = EventScope::FnEnd;
                }
                EventData::Unknown(name) => {
                    bail!("@{}(line {}) is unknown", name, position);
                }
            }
        }
        root_cmd.root.borrow().check_default_choices_fn()?;
        root_cmd.extra_args()?;
        Ok(root_cmd)
    }

    pub fn build_command(&self, name: &str) -> Result<Command> {
        let mut cmd = Command::new(name.to_string()).infer_long_args(true);
        if let Some(describe) = self.describe.as_ref() {
            cmd = cmd.about(describe);
        }
        if let Some(version) = self.version.as_ref() {
            cmd = cmd.version(version);
        }
        if let Some(author) = self.author.as_ref() {
            cmd = cmd.author(author);
        }
        if let Some(help) = self.help.as_ref() {
            cmd = cmd
                .disable_help_subcommand(true)
                .subcommand(Command::new("help").about(help))
        } else {
            cmd = cmd.disable_help_subcommand(true);
        }
        if !self.subcommands.is_empty() {
            cmd = cmd.infer_subcommands(true);
            if !self.exist_main_fn() {
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
            let subcommand = subcommand.build_command(subcommand.name.as_ref().unwrap())?;
            cmd = cmd.subcommand(subcommand);
        }
        cmd = cmd.help_template(self.help_template());
        Ok(cmd)
    }

    pub fn eval(&self, args: &[&str]) -> Result<Either<Vec<ArgcValue>, clap::Error>> {
        let name = args[0];
        if args.len() >= 2 && self.root.borrow().exist_param_fn(args[1]) {
            let mut values = vec![];
            let mut positional_args = vec![];
            if let Some(line) = args.get(2) {
                values.push(ArgcValue::Single("_line".into(), escape_shell_words(line)));
                if let Ok(words) = split_shell_words(line) {
                    let mut cur = String::new();
                    let mut escape_words: Vec<String> =
                        words.iter().map(|v| escape_shell_words(v)).collect();
                    if let Some(word) = words.last() {
                        if line.ends_with(word) {
                            cur = word.into();
                        } else {
                            escape_words.push("\"\"".into());
                        }
                    } else if !line.is_empty() {
                        escape_words.push("\"\"".into());
                    }
                    values.push(ArgcValue::Multiple("_words".into(), escape_words));
                    values.push(ArgcValue::Single("_cur".into(), cur));
                    let (args, argv) = argmap::parse(words.into_iter());
                    for (k, v) in argv {
                        let v_len = v.len();
                        match v_len {
                            0 => values.push(ArgcValue::Single(k, "1".to_string())),
                            1 => values.push(ArgcValue::Single(k, escape_shell_words(&v[0]))),
                            _ => values.push(ArgcValue::Multiple(
                                k,
                                v.iter().map(|v| escape_shell_words(v)).collect(),
                            )),
                        }
                    }
                    positional_args = args.iter().map(|v| escape_shell_words(v)).collect();
                }
            } else {
                values.push(ArgcValue::Single("_line".into(), String::new()));
                values.push(ArgcValue::Multiple("_words".into(), vec![]));
                values.push(ArgcValue::Single("_cur".into(), String::new()));
            }
            values.push(ArgcValue::ParamFn(args[1].into(), positional_args));
            return Ok(Either::Left(values));
        }
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

        for subcommand in &self.subcommands {
            if let Some(fn_name) = &subcommand.name {
                if let Some((match_name, subcommand_matches)) = matches.subcommand() {
                    if *fn_name == match_name {
                        let subcommand_values = subcommand.get_args(subcommand_matches);
                        let exist_cmd_fn = subcommand_values.iter().any(|v| v.is_cmd_fn());
                        values.extend(subcommand_values);
                        if !exist_cmd_fn {
                            values.push(ArgcValue::CmdFn(subcommand.fn_name.clone().unwrap()));
                        }
                        return values;
                    }
                }
            }
        }

        if self.exist_main_fn() {
            values.push(ArgcValue::CmdFn(self.get_main_fn()));
        }
        values
    }

    pub fn exist_main_fn(&self) -> bool {
        self.root.borrow().fns.contains_key(&self.get_main_fn())
    }

    fn get_main_fn(&self) -> String {
        match &self.name {
            Some(name) => {
                format!("{}::main", name)
            }
            None => "main".into(),
        }
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
        if self.version.is_some() {
            lines.push("{bin} {version}");
        }
        if self.author.is_some() {
            lines.push("{author}");
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

    fn get_cmd<'a>(cmd: &'a mut Self, tag_name: &str, position: usize) -> Result<&'a mut Self> {
        if cmd.root.borrow().scope == EventScope::FnEnd {
            bail!(
                "{}(line {}) is unexpected, maybe miss @cmd?",
                tag_name,
                position
            )
        }
        if cmd.subcommands.last().is_some() {
            Ok(cmd.subcommands.last_mut().unwrap())
        } else {
            Ok(cmd)
        }
    }

    fn create_subcommand(&mut self) -> &mut Self {
        let cmd = Cli {
            root: self.root.clone(),
            ..Default::default()
        };
        self.subcommands.push(cmd);
        self.subcommands.last_mut().unwrap()
    }
}

#[derive(Default)]
struct RootData {
    scope: EventScope,
    fns: HashMap<String, Position>,
    dynamic_compgen: bool,
    default_fns: Vec<(String, Position)>,
    choices_fns: Vec<(String, Position)>,
}

impl RootData {
    fn add_default_choices_fn(
        &mut self,
        position: usize,
        default_fn: &Option<String>,
        choices_fn: &Option<String>,
    ) {
        if let Some(default_fn) = default_fn.as_ref() {
            self.default_fns.push((default_fn.to_string(), position));
        }
        if let Some(choices_fn) = choices_fn.as_ref() {
            self.choices_fns.push((choices_fn.to_string(), position));
        }
    }

    fn check_default_choices_fn(&self) -> Result<()> {
        for (name, pos) in self.default_fns.iter() {
            if !self.fns.contains_key(name) {
                bail!("{}(line {}) is missing", name, pos,)
            }
        }
        for (name, pos) in self.choices_fns.iter() {
            if !self.fns.contains_key(name) {
                bail!("{}(line {}) is missing", name, pos,)
            }
        }
        Ok(())
    }

    fn exist_param_fn(&self, name: &str) -> bool {
        self.dynamic_compgen || self.choices_fns.iter().any(|(v, _)| v == name)
    }
}
