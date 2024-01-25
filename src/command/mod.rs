mod names_checker;
mod root_data;

use self::names_checker::NamesChecker;
use self::root_data::RootData;

use crate::argc_value::{ArgcValue, INIT_HOOK};
use crate::matcher::Matcher;
use crate::param::{EnvParam, FlagOptionParam, Param, PositionalParam};
use crate::parser::{parse, parse_symbol, Event, EventData, EventScope, Position};
use crate::utils::INTERNAL_MODE;
use crate::Result;

use anyhow::{anyhow, bail};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub fn eval(
    script_content: &str,
    args: &[String],
    script_path: Option<&str>,
    term_width: Option<usize>,
) -> Result<Vec<ArgcValue>> {
    let mut cmd = Command::new(script_content)?;
    cmd.eval(args, script_path, term_width)
}

pub fn export(source: &str) -> Result<serde_json::Value> {
    let cmd = Command::new(source)?;
    Ok(cmd.to_json())
}

#[derive(Debug, Default)]
pub struct Command {
    pub(crate) name: Option<String>,
    pub(crate) fn_name: Option<String>,
    pub(crate) describe: String,
    pub(crate) flag_option_params: Vec<FlagOptionParam>,
    pub(crate) positional_params: Vec<PositionalParam>,
    pub(crate) env_params: Vec<EnvParam>,
    pub(crate) subcommands: Vec<Command>,
    pub(crate) author: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) subcommand_fns: HashMap<String, Position>,
    pub(crate) alias_pos: usize,
    pub(crate) names_checker: NamesChecker,
    pub(crate) root: Arc<RefCell<RootData>>,
    pub(crate) aliases: Vec<String>,
    pub(crate) metadata: Vec<(String, String, Position)>,
    pub(crate) symbols: IndexMap<char, SymbolParam>,
}

impl Command {
    pub fn new(source: &str) -> Result<Self> {
        let events = parse(source)?;
        let mut root = Command::new_from_events(&events)?;
        if root.has_metadata("inherit-flag-options") {
            root.inherit_flag_options();
        }
        if !root.has_metadata("no-inherit-env") {
            root.inherit_envs();
        }
        Ok(root)
    }

    pub fn eval(
        &mut self,
        args: &[String],
        script_path: Option<&str>,
        term_width: Option<usize>,
    ) -> Result<Vec<ArgcValue>> {
        if args.is_empty() {
            bail!("Invalid args");
        }
        if args.len() >= 3 && args[1] == INTERNAL_MODE {
            let fallback_args = vec!["prog".to_string()];
            let new_args = if args.len() == 3 {
                &fallback_args
            } else {
                &args[3..]
            };
            let matcher = Matcher::new(self, new_args, false);
            let mut arg_values = matcher.to_arg_values_for_param_fn();
            arg_values.push(ArgcValue::ParamFn(args[2].clone()));
            return Ok(arg_values);
        }
        let mut matcher = Matcher::new(self, args, false);
        if let Some(script_path) = script_path {
            matcher.set_script_path(script_path)
        }
        if let Some(term_width) = term_width {
            matcher.set_term_width(term_width)
        }
        Ok(matcher.to_arg_values())
    }

    pub fn to_json(&self) -> serde_json::Value {
        let subcommands: Vec<serde_json::Value> =
            self.subcommands.iter().map(|v| v.to_json()).collect();
        let flag_option_params: Vec<serde_json::Value> = self
            .flag_option_params
            .iter()
            .map(|v| v.to_json())
            .collect();
        let positional_params: Vec<serde_json::Value> =
            self.positional_params.iter().map(|v| v.to_json()).collect();
        let metadata: Vec<Vec<&String>> =
            self.metadata.iter().map(|(k, v, _)| vec![k, v]).collect();
        let env_params: Vec<serde_json::Value> =
            self.env_params.iter().map(|v| v.to_json()).collect();
        serde_json::json!({
            "describe": self.describe,
            "name": self.name,
            "author": self.author,
            "version": self.version,
            "metadata": metadata,
            "options": flag_option_params,
            "positionals": positional_params,
            "aliases": self.aliases,
            "subcommands": subcommands,
            "envs": env_params,
        })
    }

    pub(crate) fn new_from_events(events: &[Event]) -> Result<Self> {
        let mut root_cmd = Command::default();
        let root_data = root_cmd.root.clone();
        for event in events {
            let Event { data, position } = event.clone();
            match data {
                EventData::Describe(value) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@describe", position)?;
                    cmd.describe = value;
                }
                EventData::Version(value) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@version", position)?;
                    cmd.version = Some(value);
                }
                EventData::Author(value) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@author", position)?;
                    cmd.author = Some(value);
                }
                EventData::Meta(key, value) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@meta", position)?;
                    if key == "symbol" {
                        let (ch, name, choice_fn) = parse_symbol(&value).ok_or_else(|| {
                            anyhow!("@meta(line {}) invalid symbol value", position)
                        })?;
                        cmd.symbols
                            .insert(ch, (name.to_string(), choice_fn.map(|v| v.to_string())));
                    }
                    cmd.metadata.push((key, value, position));
                }
                EventData::Cmd(value) => {
                    if root_data.borrow().scope == EventScope::CmdStart {
                        bail!(
                            "@cmd(line {}) missing function?",
                            root_data.borrow().cmd_pos
                        )
                    }
                    root_data.borrow_mut().cmd_pos = position;
                    root_data.borrow_mut().scope = EventScope::CmdStart;
                    let subcmd = root_cmd.create_cmd();
                    if !value.is_empty() {
                        subcmd.describe = value.clone();
                    }
                }
                EventData::Aliases(values) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@alias", position)?;
                    cmd.alias_pos = position;
                    cmd.aliases = values.to_vec();
                }
                EventData::FlagOption(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    if param.is_option() {
                        root_data.borrow_mut().add_param_fn(
                            position,
                            param.default_fn(),
                            param.choice_fn(),
                        );
                    }
                    cmd.names_checker.check_flag_option(&param, position)?;
                    cmd.flag_option_params.push(param);
                }
                EventData::Env(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    root_data.borrow_mut().add_param_fn(
                        position,
                        param.default_fn(),
                        param.choice_fn(),
                    );
                    cmd.names_checker.check_env(&param, position)?;
                    cmd.env_params.push(param);
                }
                EventData::Positional(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    root_data.borrow_mut().add_param_fn(
                        position,
                        param.default_fn(),
                        param.choice_fn(),
                    );
                    cmd.add_positional_param(param, position)?;
                }
                EventData::Func(name) => {
                    if let Some(pos) = root_data.borrow_mut().cmd_fns.get(&name) {
                        bail!(
                            "{}(line {}) conflicts with cmd or alias at line {}",
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

                        let parts: Vec<&str> = name.split("::").collect();
                        let parts_len = parts.len();
                        if parts_len == 0 {
                            bail!("{}(line {}) invalid command name", name, position);
                        } else if parts_len == 1 {
                            let cmd = root_cmd.subcommands.last_mut().unwrap();
                            cmd.name = Some(sanitize_cmd_name(&name));
                            cmd.fn_name = Some(name.to_string());
                            for name in &cmd.aliases {
                                if let Some(pos) = root_data.borrow().cmd_fns.get(name) {
                                    bail!(
                                        "@alias(line {}) conflicts with cmd or alias at line {}",
                                        cmd.alias_pos,
                                        pos
                                    );
                                }
                                root_data
                                    .borrow_mut()
                                    .cmd_fns
                                    .insert(name.clone(), cmd.alias_pos);
                            }
                        } else {
                            let mut cmd = root_cmd.subcommands.pop().unwrap();
                            let (child, parents) = parts.split_last().unwrap();
                            let parents: Vec<String> =
                                parents.iter().map(|v| sanitize_cmd_name(v)).collect();
                            cmd.name = Some(sanitize_cmd_name(child));
                            cmd.fn_name = Some(name.to_string());
                            match retrive_cmd(&mut root_cmd, &parents) {
                                Some(parent_cmd) => {
                                    parent_cmd
                                        .subcommand_fns
                                        .insert(child.to_string(), position);
                                    for name in &cmd.aliases {
                                        if let Some(pos) = parent_cmd.subcommand_fns.get(name) {
                                            bail!(
												"@alias(line {}) conflicts with cmd or alias at line {}",
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
                                    bail!("{}(line {}) lack of parent command", name, position);
                                }
                            }
                        }
                    }
                    root_data.borrow_mut().scope = EventScope::FnEnd;
                }
                EventData::Unknown(name) => {
                    bail!("@{}(line {}) is unknown tag", name, position);
                }
            }
        }
        root_cmd.root.borrow().check_param_fn()?;
        Ok(root_cmd)
    }

    pub(crate) fn has_metadata(&self, key: &str) -> bool {
        self.metadata.iter().any(|(k, _, _)| k == key)
    }

    pub(crate) fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, v, _)| v)
    }

    pub(crate) fn flag_option_signs(&self) -> String {
        let mut signs = HashSet::new();
        signs.insert('-');
        for param in &self.flag_option_params {
            if let Some(short) = &param.short {
                signs.extend(short.chars().take(1))
            }
            signs.extend(param.long_prefix.chars())
        }
        signs.into_iter().collect()
    }

    pub(crate) fn render_help(&self, cmd_paths: &[&str], term_width: Option<usize>) -> String {
        let mut output = vec![];
        if self.version.is_some() {
            output.push(self.render_version(cmd_paths));
        }
        if let Some(author) = &self.author {
            output.push(author.to_string());
        }
        if !&self.describe.is_empty() {
            output.push(wrap_render_block("", &self.describe, term_width));
        }
        if !output.is_empty() {
            output.push(String::new());
        }
        output.push(self.render_usage(cmd_paths));
        output.push(String::new());
        output.extend(self.render_positionals(term_width));
        output.extend(self.render_flag_options(term_width));
        output.extend(self.render_subcommands(term_width));
        output.extend(self.render_envs(term_width));
        if output.is_empty() {
            return "\n".to_string();
        }
        output.join("\n")
    }

    pub(crate) fn render_version(&self, cmd_paths: &[&str]) -> String {
        format!(
            "{} {}",
            cmd_paths.join("-"),
            self.version.clone().unwrap_or_else(|| "0.0.0".to_string())
        )
    }

    pub(crate) fn render_usage(&self, cmd_paths: &[&str]) -> String {
        let mut output = vec!["USAGE:".to_string()];
        output.extend(cmd_paths.iter().map(|v| v.to_string()));
        let required_options: Vec<String> = self
            .flag_option_params
            .iter()
            .filter(|v| v.required())
            .map(|v| v.render_name_notations())
            .collect();
        if self.flag_option_params.len() != required_options.len() {
            output.push("[OPTIONS]".to_string());
        }
        output.extend(required_options);
        output.extend(self.positional_params.iter().map(|v| v.render_value()));
        if !self.subcommands.is_empty() {
            output.push("<COMMAND>".to_string());
        }
        output.join(" ")
    }

    pub(crate) fn render_positionals(&self, term_width: Option<usize>) -> Vec<String> {
        let mut output = vec![];
        if self.positional_params.is_empty() {
            return output;
        }
        let mut list = vec![];
        let mut value_size = 0;
        for param in self.positional_params.iter() {
            let value = param.render_value();
            value_size = value_size.max(value.len());
            list.push((value, param.render_describe()));
        }
        output.push("ARGS:".to_string());
        value_size += 2;
        for (value, describe) in list {
            if describe.is_empty() {
                output.push(format!("  {value}"));
            } else {
                let spaces = " ".repeat(value_size - value.len());
                output.push(wrap_render_block(
                    &format!("  {value}{spaces}"),
                    &describe,
                    term_width,
                ));
            }
        }
        output.push("".to_string());
        output
    }

    pub(crate) fn render_flag_options(&self, term_width: Option<usize>) -> Vec<String> {
        let mut output = vec![];
        if self.flag_option_params.is_empty() {
            return output;
        }
        let mut list = vec![];
        let mut any_describe = false;
        let mut single = false;
        for param in self.flag_option_params.iter() {
            if param.long_prefix.len() == 1 {
                single = true;
            }
            let value = param.render_body();
            let describe = param.render_describe();
            if !describe.is_empty() {
                any_describe = true;
            }
            list.push((value, describe));
        }
        self.add_help_flag(&mut list, single, any_describe);
        self.add_version_flag(&mut list, single, any_describe);
        output.push("OPTIONS:".to_string());
        let value_size = list.iter().map(|v| v.0.len()).max().unwrap_or_default() + 2;
        for (value, describe) in list {
            if describe.is_empty() {
                output.push(format!("  {value}"));
            } else {
                let spaces = " ".repeat(value_size - value.len());
                output.push(wrap_render_block(
                    &format!("  {value}{spaces}"),
                    &describe,
                    term_width,
                ));
            }
        }
        output.push("".to_string());
        output
    }

    pub(crate) fn render_subcommands(&self, term_width: Option<usize>) -> Vec<String> {
        let mut output = vec![];
        if self.subcommands.is_empty() {
            return output;
        }
        let mut list = vec![];
        let mut value_size = 0;
        for cmd in self.subcommands.iter() {
            let value = cmd.name.clone().unwrap_or_default();
            let describe = cmd.render_subcommand_describe();
            value_size = value_size.max(value.len());
            list.push((value, describe));
        }
        output.push("COMMANDS:".to_string());
        value_size += 2;
        for (value, describe) in list {
            if describe.is_empty() {
                output.push(format!("  {value}"));
            } else {
                let spaces = " ".repeat(value_size - value.len());
                output.push(wrap_render_block(
                    &format!("  {value}{spaces}"),
                    &describe,
                    term_width,
                ));
            }
        }
        output.push("".to_string());
        output
    }

    pub(crate) fn render_subcommand_describe(&self) -> String {
        let mut output = self.describe_oneline().to_string();
        if self.aliases.is_empty() {
            return output;
        } else {
            if !output.is_empty() {
                output.push(' ')
            }
            output.push_str(&format!("[aliases: {}]", self.aliases.join(", ")));
        };
        output
    }

    pub(crate) fn render_envs(&self, term_width: Option<usize>) -> Vec<String> {
        let mut output = vec![];
        if self.env_params.is_empty() {
            return output;
        }
        let mut list = vec![];
        for param in self.env_params.iter() {
            let value = param.render_body();
            let describe = param.render_describe();
            list.push((value, describe));
        }
        output.push("ENVIRONMENTS:".to_string());
        let value_size = list.iter().map(|v| v.0.len()).max().unwrap_or_default() + 2;
        for (value, describe) in list {
            if describe.is_empty() {
                output.push(format!("  {value}"));
            } else {
                let spaces = " ".repeat(value_size - value.len());
                output.push(wrap_render_block(
                    &format!("  {value}{spaces}"),
                    &describe,
                    term_width,
                ));
            }
        }
        output.push("".to_string());
        output
    }

    pub(crate) fn describe_oneline(&self) -> &str {
        match self.describe.split_once('\n') {
            Some((v, _)) => v,
            None => self.describe.as_str(),
        }
    }

    pub(crate) fn list_names(&self) -> Vec<String> {
        let mut output: Vec<String> = self.name.clone().into_iter().collect();
        output.extend(self.aliases.to_vec());
        output
    }

    pub(crate) fn list_subcommand_names(&self) -> Vec<String> {
        self.subcommands
            .iter()
            .flat_map(|v| v.list_names())
            .collect()
    }

    pub(crate) fn find_subcommand(&self, name: &str) -> Option<&Self> {
        self.subcommands.iter().find(|subcmd| {
            if let Some(subcmd_name) = &subcmd.name {
                if subcmd_name == name {
                    return true;
                }
            }
            return subcmd.aliases.iter().any(|v| v == name);
        })
    }

    pub(crate) fn find_flag_option(&self, name: &str) -> Option<&FlagOptionParam> {
        self.flag_option_params.iter().find(|v| v.is_match(name))
    }

    pub(crate) fn find_prefixed_option(&self, name: &str) -> Option<(&FlagOptionParam, String)> {
        for param in self.flag_option_params.iter() {
            if let Some(prefix) = param.prefixed() {
                if name.starts_with(&prefix) {
                    return Some((param, prefix));
                }
            }
        }
        None
    }

    pub(crate) fn find_env(&self, name: &str) -> Option<&EnvParam> {
        self.env_params.iter().find(|v| v.var_name() == name)
    }

    pub(crate) fn match_version_short_name(&self) -> bool {
        match self.find_flag_option("-V") {
            Some(param) => param.var_name() == "version",
            None => true,
        }
    }

    pub(crate) fn match_help_short_name(&self) -> bool {
        match self.find_flag_option("-h") {
            Some(param) => param.var_name() == "help",
            None => true,
        }
    }

    pub(crate) fn no_flags_options_subcommands(&self) -> bool {
        self.flag_option_params.is_empty() && self.subcommands.is_empty()
    }

    pub(crate) fn get_cmd_fn(&self, cmd_paths: &[&str]) -> Option<String> {
        let main = "main".to_string();
        if cmd_paths.len() < 2 {
            if self.root.borrow().fns.contains_key(&main) {
                Some(main)
            } else {
                None
            }
        } else if self.subcommands.is_empty() {
            self.fn_name.clone()
        } else {
            let mut parts: Vec<String> = cmd_paths.iter().skip(1).map(|v| v.to_string()).collect();
            parts.push(main);
            let name = parts.join("::");
            if self.root.borrow().fns.contains_key(&name) {
                Some(name)
            } else {
                None
            }
        }
    }

    pub(crate) fn exist_init_hook(&self) -> bool {
        self.root.borrow().fns.contains_key(INIT_HOOK)
    }

    pub(crate) fn delegated(&self) -> bool {
        self.subcommands.is_empty()
            && self.flag_option_params.is_empty()
            && self.positional_params.len() == 1
            && self
                .positional_params
                .first()
                .map(|v| v.terminated())
                .unwrap_or_default()
    }

    pub(crate) fn exist_main_fn(&self, cmd_paths: &[&str]) -> bool {
        self.get_cmd_fn(cmd_paths)
            .map(|v| v.ends_with("main"))
            .unwrap_or_default()
    }

    fn inherit_flag_options(&mut self) {
        for subcmd in self.subcommands.iter_mut() {
            let mut inherited_flag_options = vec![];
            for flag_option in &self.flag_option_params {
                if subcmd.find_flag_option(flag_option.var_name()).is_none() {
                    let mut flag_option = flag_option.clone();
                    flag_option.inherit = true;
                    inherited_flag_options.push(flag_option);
                }
            }
            subcmd
                .flag_option_params
                .splice(..0, inherited_flag_options);
        }
        for subcmd in self.subcommands.iter_mut() {
            subcmd.inherit_flag_options();
        }
    }

    fn inherit_envs(&mut self) {
        for subcmd in self.subcommands.iter_mut() {
            let mut inherited_envs = vec![];
            for env_param in &self.env_params {
                if subcmd.find_env(env_param.var_name()).is_none() {
                    let mut env_param = env_param.clone();
                    env_param.inherit = true;
                    inherited_envs.push(env_param);
                }
            }
            subcmd.env_params.splice(..0, inherited_envs);
        }
        for subcmd in self.subcommands.iter_mut() {
            subcmd.inherit_envs();
        }
    }

    fn add_positional_param(&mut self, param: PositionalParam, pos: Position) -> Result<()> {
        self.names_checker.check_positional(&param, pos)?;
        self.positional_params.push(param);
        Ok(())
    }

    fn get_cmd<'a>(cmd: &'a mut Self, tag_name: &str, position: usize) -> Result<&'a mut Self> {
        if cmd.root.borrow().scope == EventScope::FnEnd {
            bail!(
                "{}(line {}) shouldn't be here, @cmd is missing?",
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

    fn create_cmd(&mut self) -> &mut Self {
        let cmd = Command {
            root: self.root.clone(),
            names_checker: Default::default(),
            ..Default::default()
        };
        self.subcommands.push(cmd);
        self.subcommands.last_mut().unwrap()
    }

    fn add_help_flag(&self, list: &mut Vec<(String, String)>, single: bool, any_describe: bool) {
        if self.find_flag_option("help").is_some() {
            return;
        }
        let hyphens = if single { " -" } else { "--" };
        list.push((
            if self.match_help_short_name() {
                format!("-h, {}help", hyphens)
            } else {
                format!("    {}help", hyphens)
            },
            if any_describe {
                "Print help".into()
            } else {
                "".into()
            },
        ));
    }

    fn add_version_flag(&self, list: &mut Vec<(String, String)>, single: bool, any_describe: bool) {
        if self.version.is_none() {
            return;
        }
        if self.find_flag_option("version").is_some() {
            return;
        }
        let hyphens = if single { " -" } else { "--" };
        list.push((
            if self.match_version_short_name() {
                format!("-V, {}version", hyphens)
            } else {
                format!("    {}version", hyphens)
            },
            if any_describe {
                "Print version".into()
            } else {
                "".into()
            },
        ));
    }
}

pub(crate) type SymbolParam = (String, Option<String>);

fn retrive_cmd<'a>(cmd: &'a mut Command, cmd_paths: &[String]) -> Option<&'a mut Command> {
    if cmd_paths.is_empty() {
        return Some(cmd);
    }
    let child = cmd
        .subcommands
        .iter_mut()
        .find(|v| v.name.as_deref() == Some(cmd_paths[0].as_str()))?;
    retrive_cmd(child, &cmd_paths[1..])
}

fn sanitize_cmd_name(name: &str) -> String {
    name.trim_end_matches('_').to_string()
}

fn wrap_render_block(name: &str, describe: &str, term_width: Option<usize>) -> String {
    let size = term_width.unwrap_or(999) - name.len();
    let empty = " ".repeat(name.len());
    describe
        .split('\n')
        .flat_map(|v| textwrap::wrap(v, size))
        .enumerate()
        .map(|(i, v)| {
            if i == 0 {
                format!("{name}{v}")
            } else {
                format!("{empty}{v}")
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}
