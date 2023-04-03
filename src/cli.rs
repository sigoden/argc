use crate::argc_value::ArgcValue;
use crate::param::{Param, ParamNames, PositionalParam};
use crate::parser::{parse, Event, EventData, EventScope, Position};
use crate::utils::{escape_shell_words, split_shell_words};
use crate::Result;
use anyhow::{bail, Context};
use clap::{ArgMatches, Command};
use either::Either;
use std::cell::RefCell;
use std::collections::HashMap;
use std::result::Result as StdResult;
use std::sync::Arc;

pub fn eval(source: &str, args: &[&str]) -> Result<Either<String, clap::Error>> {
    let events = parse(source)?;
    let cmd = Cli::new_from_events(&events)?;
    match cmd.eval(args)? {
        Either::Left(values) => Ok(Either::Left(ArgcValue::to_shell(values))),
        Either::Right(error) => Ok(Either::Right(error)),
    }
}

pub fn export(source: &str, name: &str) -> Result<serde_json::Value> {
    let events = parse(source)?;
    let mut cmd = Cli::new_from_events(&events)?;
    cmd.name = Some(name.to_string());
    cmd.to_json().with_context(|| "Failed to export json")
}

#[derive(Default)]
pub struct Cli {
    name: Option<String>,
    fn_name: Option<String>,
    describe: Option<String>,
    params: Vec<Param>,
    positional_pos: Vec<Position>,
    subcommands: Vec<Cli>,
    help: Option<String>,
    author: Option<String>,
    version: Option<String>,
    subcommand_fns: HashMap<String, Position>,
    alias_pos: usize,
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
                    if root_data.borrow().scope == EventScope::CmdStart {
                        bail!("@cmd(line {}) miss function?", root_data.borrow().cmd_pos)
                    }
                    root_data.borrow_mut().cmd_pos = position;
                    root_data.borrow_mut().scope = EventScope::CmdStart;
                    let mut subcmd = root_cmd.create_subcommand();
                    if !value.is_empty() {
                        subcmd.describe = Some(value.clone());
                    }
                }
                EventData::Aliases(values) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@alias", position)?;
                    cmd.alias_pos = position;
                    cmd.aliases = values.to_vec();
                }
                EventData::Flag(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    param.detect_conflict(&mut cmd.names, position)?;
                    cmd.params.push(Param::Flag(param));
                }
                EventData::Option(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    root_data.borrow_mut().add_default_choices_fn(
                        position,
                        &param.default_fn,
                        &param.choices_fn,
                    );
                    param.detect_conflict(&mut cmd.names, position)?;
                    cmd.params.push(Param::Option(param));
                }
                EventData::Positional(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    root_data.borrow_mut().add_default_choices_fn(
                        position,
                        &param.default_fn,
                        &param.choices_fn,
                    );
                    cmd.add_positional_param(param, position)?;
                }
                EventData::Func(name) => {
                    if let Some(pos) = root_data.borrow_mut().cmd_fns.get(&name) {
                        bail!(
                            "{}(line {}) is conflicted with cmd or alias at line {}",
                            name,
                            position,
                            pos
                        )
                    }
                    root_data.borrow_mut().fns.insert(name.clone(), position);
                    if root_data.borrow().scope == EventScope::CmdStart {
                        root_data
                            .borrow_mut()
                            .cmd_fns
                            .insert(name.clone(), position);
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
                                for name in &cmd.aliases {
                                    if let Some(pos) = root_data.borrow().cmd_fns.get(name) {
                                        bail!(
											"@alias(line {}) is conflicted with cmd or alias at line {}",
											cmd.alias_pos,
											pos
										);
                                    }
                                    root_data
                                        .borrow_mut()
                                        .cmd_fns
                                        .insert(name.clone(), cmd.alias_pos);
                                }
                            }
                            Some(child) => {
                                let mut cmd = root_cmd.subcommands.pop().unwrap();
                                cmd.name = Some(child.to_string());
                                cmd.fn_name = Some(name.to_string());
                                match root_cmd
                                    .subcommands
                                    .iter_mut()
                                    .find(|v| v.name == Some(parent.into()))
                                {
                                    Some(parent_cmd) => {
                                        parent_cmd
                                            .subcommand_fns
                                            .insert(child.to_string(), position);
                                        for name in &cmd.aliases {
                                            if let Some(pos) = parent_cmd.subcommand_fns.get(name) {
                                                bail!(
													"@alias(line {}) is conflicted with cmd or alias at line {}",
													cmd.alias_pos,
													pos
												);
                                            }
                                            parent_cmd
                                                .subcommand_fns
                                                .insert(name.clone(), cmd.alias_pos);
                                        }
                                        parent_cmd.subcommands.push(cmd);
                                    }
                                    None => {
                                        bail!("{}(line {}) has no parent", name, position);
                                    }
                                }
                            }
                        }
                    }
                    root_data.borrow_mut().scope = EventScope::FnEnd;
                }
                EventData::Unknown(name) => {
                    bail!("@{}(line {}) is unknown", name, position);
                }
            }
        }
        root_cmd.root.borrow().check_default_choices_fn()?;
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
        let mut positional_params = vec![];
        for param in &self.params {
            let arg = match param {
                Param::Flag(param) => param.build_arg()?,
                Param::Option(param) => param.build_arg()?,
                Param::Positional(param) => {
                    positional_params.push(param);
                    continue;
                }
            };
            cmd = cmd.arg(arg);
        }
        if positional_params.is_empty() {
            let mut arg = PositionalParam::extra().build_arg(0)?;
            arg = arg.allow_hyphen_values(true).trailing_var_arg(true);
            cmd = cmd.arg(arg);
        } else {
            for (index, param) in positional_params.iter().enumerate() {
                cmd = cmd.arg(param.build_arg(index)?);
            }
        }
        for subcommand in &self.subcommands {
            let subcommand = subcommand.build_command(subcommand.name.as_ref().unwrap())?;
            cmd = cmd.subcommand(subcommand);
        }
        cmd = cmd.help_template(self.help_template());
        Ok(cmd)
    }

    pub fn eval(&self, args: &[&str]) -> Result<Either<Vec<ArgcValue>, clap::Error>> {
        if args.len() >= 2 && self.root.borrow().exist_param_fn(args[1]) {
            return self.eval_param_fn(args);
        }
        let command = self.build_command(args[0])?;
        let res = command.try_get_matches_from(args);
        match res {
            Ok(matches) => {
                let values = self.get_args(&matches);
                Ok(Either::Left(values))
            }
            Err(err) => Ok(Either::Right(err)),
        }
    }

    pub fn to_json(&self) -> StdResult<serde_json::Value, serde_json::Error> {
        let subcommands: StdResult<Vec<serde_json::Value>, _> =
            self.subcommands.iter().map(|v| v.to_json()).collect();
        let params: StdResult<Vec<serde_json::Value>, _> =
            self.params.iter().map(serde_json::to_value).collect();
        let value = serde_json::json!({
            "describe": self.describe,
            "name": self.name,
            "help": self.help,
            "author": self.author,
            "version": self.version,
            "params": params?,
            "aliases": self.aliases,
            "subcommands": subcommands?,
        });
        Ok(value)
    }

    fn eval_param_fn(&self, args: &[&str]) -> Result<Either<Vec<ArgcValue>, clap::Error>> {
        let mut values = vec![];
        if let Some(line) = args.get(2) {
            values.push(ArgcValue::Single("_line".into(), escape_shell_words(line)));
            if let Ok(words) = split_shell_words(line) {
                let mut escape_words: Vec<String> =
                    words.iter().map(|v| escape_shell_words(v)).collect();
                if let Some(word) = words.last() {
                    if !line.ends_with(word) {
                        escape_words.push(escape_shell_words(" "));
                    }
                } else if !line.is_empty() {
                    escape_words.push(escape_shell_words(" "));
                }
                values.push(ArgcValue::Multiple("_words".into(), escape_words));
                if !words.is_empty()
                    && (words[0].starts_with('-') || self.exist_subcommand(&words[0]))
                {
                    let (left, right) = if let Some(index) = words.iter().position(|s| s == "--") {
                        words.split_at(index)
                    } else {
                        words.split_at(words.len())
                    };
                    if let Ok(command) = self.build_command_loose(args[0]) {
                        let mut positional_args = vec![];
                        let mut mathc_args = left.to_vec();
                        mathc_args.insert(0, mathc_args[0].to_string());
                        if let Ok(matches) = command.try_get_matches_from(&mathc_args) {
                            for value in self.get_args(&matches) {
                                if value.is_cmd_fn() {
                                    continue;
                                }
                                if let ArgcValue::PositionalMultiple(_, args) = value {
                                    positional_args = args;
                                    continue;
                                }
                                values.push(value);
                            }
                        }
                        if !right.is_empty() {
                            values.push(ArgcValue::Single(
                                "_dashdash".into(),
                                positional_args.len().to_string(),
                            ));
                            positional_args.extend(right[1..].iter().map(|v| v.to_string()));
                        }
                        values.push(ArgcValue::PositionalMultiple(
                            "_args".into(),
                            positional_args,
                        ));
                    }
                }
            }
        } else {
            values.push(ArgcValue::Single("_line".into(), String::new()));
            values.push(ArgcValue::Multiple("_words".into(), vec![]));
        }
        values.push(ArgcValue::ParamFn(args[1].into()));
        Ok(Either::Left(values))
    }

    fn build_command_loose(&self, name: &str) -> Result<Command> {
        let mut cmd = Command::new(name.to_string());
        cmd = cmd.ignore_errors(true);
        if let Some(help) = self.help.as_ref() {
            cmd = cmd
                .disable_help_subcommand(true)
                .subcommand(Command::new("help").about(help))
        } else {
            cmd = cmd.disable_help_subcommand(true);
        }
        if !self.aliases.is_empty() {
            cmd = cmd.visible_aliases(&self.aliases);
        }
        for param in &self.params {
            let arg = match param {
                Param::Flag(param) => param.build_arg()?,
                Param::Option(param) => param.build_arg_loose()?,
                Param::Positional(_) => {
                    continue;
                }
            };
            cmd = cmd.arg(arg);
        }
        cmd = cmd.arg(PositionalParam::extra().build_arg(0)?);
        for subcommand in &self.subcommands {
            let subcommand = subcommand.build_command(subcommand.name.as_ref().unwrap())?;
            cmd = cmd.subcommand(subcommand);
        }
        Ok(cmd)
    }

    fn get_args(&self, matches: &ArgMatches) -> Vec<ArgcValue> {
        let mut values = vec![];
        let mut no_positional_param = true;
        for param in &self.params {
            match param {
                Param::Flag(param) => {
                    if let Some(value) = param.get_arg_value(matches) {
                        values.push(value);
                    }
                }
                Param::Option(param) => {
                    if let Some(value) = param.get_arg_value(matches) {
                        values.push(value);
                    }
                }
                Param::Positional(param) => {
                    if let Some(value) = param.get_arg_value(matches) {
                        values.push(value);
                    }
                    no_positional_param = false;
                }
            }
        }
        if no_positional_param {
            if let Some(value) = PositionalParam::extra().get_arg_value(matches) {
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

    fn exist_main_fn(&self) -> bool {
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

    fn exist_subcommand(&self, name: &str) -> bool {
        self.subcommands.iter().any(|subcmd| {
            if let Some(subcmd_name) = &subcmd.name {
                if subcmd_name == name {
                    return true;
                }
            }
            return subcmd.aliases.iter().any(|v| v == name);
        })
    }

    fn add_positional_param(&mut self, param: PositionalParam, pos: Position) -> Result<()> {
        param.detect_conflict(&mut self.names, pos)?;
        self.params.push(Param::Positional(param));
        self.positional_pos.push(pos);
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
            .any(|v| matches!(v, Param::Positional(_)));
        let has_options = self
            .params
            .iter()
            .any(|v| matches!(v, Param::Flag(_) | Param::Option(_)));
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
    cmd_fns: HashMap<String, Position>,
    cmd_pos: usize,
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
        self.choices_fns.iter().any(|(v, _)| v == name)
    }
}
