mod names_checker;
mod root_data;

use self::names_checker::NamesChecker;
use self::root_data::RootData;

use crate::argc_value::ArgcValue;
use crate::matcher::Matcher;
use crate::param::{FlagOptionParam, PositionalParam};
use crate::parser::{parse, Event, EventData, EventScope, Position};
use crate::utils::split_shell_words;
use crate::Result;

use anyhow::{bail, Context};
use std::cell::RefCell;
use std::collections::HashMap;
use std::result::Result as StdResult;
use std::sync::Arc;

pub fn eval(
    script_path: Option<&str>,
    script_content: &str,
    args: &[String],
    term_width: Option<usize>,
) -> Result<Vec<ArgcValue>> {
    let mut cmd = Command::new(script_content)?;
    cmd.eval(script_path, args, term_width)
}

pub fn export(source: &str) -> Result<serde_json::Value> {
    let cmd = Command::new(source)?;
    cmd.to_json().with_context(|| "Failed to export json")
}

#[derive(Default)]
pub struct Command {
    pub(crate) name: Option<String>,
    pub(crate) fn_name: Option<String>,
    pub(crate) describe: String,
    pub(crate) flag_option_params: Vec<FlagOptionParam>,
    pub(crate) positional_params: Vec<PositionalParam>,
    pub(crate) positional_pos: Vec<Position>,
    pub(crate) subcommands: Vec<Command>,
    pub(crate) author: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) subcommand_fns: HashMap<String, Position>,
    pub(crate) alias_pos: usize,
    pub(crate) names_checker: NamesChecker,
    pub(crate) root: Arc<RefCell<RootData>>,
    pub(crate) aliases: Vec<String>,
}

impl Command {
    pub fn new(source: &str) -> Result<Self> {
        let events = parse(source)?;
        Command::new_from_events(&events)
    }

    pub fn eval(
        &mut self,
        script_path: Option<&str>,
        args: &[String],
        term_width: Option<usize>,
    ) -> Result<Vec<ArgcValue>> {
        if args.is_empty() {
            bail!("Invalid args");
        }
        if args.len() >= 2 && self.root.borrow().exist_param_fn(args[1].as_str()) {
            let mut arg_values = vec![];
            let (line, mut words) = match args.get(2) {
                Some(line) => {
                    let mut words = split_shell_words(line).ok().unwrap_or_default();
                    words.insert(0, args[0].clone());
                    (line.clone(), words)
                }
                None => (String::new(), vec![]),
            };
            if !words.is_empty() {
                if line.trim_end() != line {
                    words.push(" ".into());
                }
                let matcher = Matcher::new(self, &words);
                arg_values.extend(matcher.to_arg_values_for_choice_fn());
            }
            arg_values.push(ArgcValue::Single("_line".into(), line));
            arg_values.push(ArgcValue::Multiple("_words".into(), words));
            arg_values.push(ArgcValue::ParamFn(args[1].clone()));
            return Ok(arg_values);
        }
        let mut matcher = Matcher::new(self, args);
        if let Some(script_path) = script_path {
            matcher.set_script_path(script_path)
        }
        if let Some(term_width) = term_width {
            matcher.set_term_width(term_width)
        }
        Ok(matcher.to_arg_values())
    }

    pub fn to_json(&self) -> StdResult<serde_json::Value, serde_json::Error> {
        let subcommands: StdResult<Vec<serde_json::Value>, _> =
            self.subcommands.iter().map(|v| v.to_json()).collect();
        let flag_option_params: StdResult<Vec<serde_json::Value>, _> = self
            .flag_option_params
            .iter()
            .map(serde_json::to_value)
            .collect();
        let positional_params: StdResult<Vec<serde_json::Value>, _> = self
            .positional_params
            .iter()
            .map(serde_json::to_value)
            .collect();
        let value = serde_json::json!({
            "describe": self.describe,
            "name": self.name,
            "author": self.author,
            "version": self.version,
            "options": flag_option_params?,
            "positionals": positional_params?,
            "aliases": self.aliases,
            "subcommands": subcommands?,
        });
        Ok(value)
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
                EventData::Cmd(value) => {
                    if root_data.borrow().scope == EventScope::CmdStart {
                        bail!("@cmd(line {}) miss function?", root_data.borrow().cmd_pos)
                    }
                    root_data.borrow_mut().cmd_pos = position;
                    root_data.borrow_mut().scope = EventScope::CmdStart;
                    let mut subcmd = root_cmd.create_cmd();
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
                            &param.default_fn,
                            &param.choices_fn,
                        );
                    }
                    cmd.names_checker.check_flag_option(&param, position)?;
                    cmd.flag_option_params.push(param);
                }
                EventData::Positional(param) => {
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    root_data.borrow_mut().add_param_fn(
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

                        let parts: Vec<&str> = name.split("::").collect();
                        let parts_len = parts.len();
                        if parts_len == 0 {
                            bail!("{}(line {}) invalid command name", name, position);
                        } else if parts_len == 1 {
                            let cmd = root_cmd.subcommands.last_mut().unwrap();
                            cmd.name = Some(parts[0].to_string());
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
                        } else {
                            let mut cmd = root_cmd.subcommands.pop().unwrap();
                            let (child, parents) = parts.split_last().unwrap();
                            cmd.name = Some(child.to_string());
                            cmd.fn_name = Some(name.to_string());
                            match retrive_cmd(&mut root_cmd, parents) {
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
                                    bail!("{}(line {}) lack of parent command", name, position);
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
        root_cmd.root.borrow().check_param_fn()?;
        Ok(root_cmd)
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
            .filter(|v| v.required)
            .map(|v| v.render_name_values())
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
        let mut any_summmary = false;
        let mut double_help_dashes = true;
        for param in self.flag_option_params.iter() {
            if param.dashes == "-" {
                double_help_dashes = false;
            }
            let value = param.render_body();
            let describe = param.render_describe();
            if !describe.is_empty() {
                any_summmary = true;
            }
            list.push((value, describe));
        }
        let help_dashes = if double_help_dashes { "--" } else { " -" };
        list.push((
            if self.match_help_short_name() {
                format!("-h, {}help", help_dashes)
            } else {
                format!("    {}help", help_dashes)
            },
            if any_summmary {
                "Print help".into()
            } else {
                "".into()
            },
        ));
        if self.version.is_some() {
            list.push((
                if self.match_version_short_name() {
                    "    --version".into()
                } else {
                    "-V, --version".into()
                },
                if any_summmary {
                    "Print version".into()
                } else {
                    "".into()
                },
            ));
        }
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
        let mut output = match self.describe.split_once('\n') {
            Some((v, _)) => v.to_string(),
            None => self.describe.clone(),
        };
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

    pub(crate) fn list_names(&self) -> Vec<String> {
        let mut output = vec![self.name.clone().unwrap_or_default()];
        output.extend(self.aliases.to_vec());
        output
    }

    pub(crate) fn list_subcommand_names(&self) -> Vec<String> {
        let mut output = vec![];
        for subcmd in self.subcommands.iter() {
            if let Some(name) = subcmd.name.clone() {
                output.push(name);
            }
            output.extend(subcmd.aliases.to_vec());
        }
        output
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
        self.flag_option_params
            .iter()
            .find(|v| v.name == name || v.is_match(name))
    }

    pub(crate) fn match_version_short_name(&self) -> bool {
        match self.find_flag_option("-V") {
            Some(param) => &param.name == "version",
            None => true,
        }
    }

    pub(crate) fn match_help_short_name(&self) -> bool {
        match self.find_flag_option("-h") {
            Some(param) => &param.name == "help",
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

    pub(crate) fn exist_main_fn(&self, cmd_paths: &[&str]) -> bool {
        self.get_cmd_fn(cmd_paths)
            .map(|v| v.ends_with("main"))
            .unwrap_or_default()
    }

    fn add_positional_param(&mut self, param: PositionalParam, pos: Position) -> Result<()> {
        self.names_checker.check_positional(&param, pos)?;
        self.positional_params.push(param);
        self.positional_pos.push(pos);
        Ok(())
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

    fn create_cmd(&mut self) -> &mut Self {
        let cmd = Command {
            root: self.root.clone(),
            names_checker: Default::default(),
            ..Default::default()
        };
        self.subcommands.push(cmd);
        self.subcommands.last_mut().unwrap()
    }
}

fn retrive_cmd<'a>(cmd: &'a mut Command, cmd_paths: &[&str]) -> Option<&'a mut Command> {
    if cmd_paths.is_empty() {
        return Some(cmd);
    }
    let child = cmd
        .subcommands
        .iter_mut()
        .find(|v| v.name.as_deref() == Some(cmd_paths[0]))?;
    retrive_cmd(child, &cmd_paths[1..])
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
