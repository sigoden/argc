mod names_checker;
mod share_data;

use self::names_checker::NamesChecker;
use self::share_data::ShareData;

use crate::argc_value::ArgcValue;
#[cfg(feature = "eval")]
use crate::matcher::Matcher;
use crate::param::{EnvParam, FlagOptionParam, Param, PositionalParam};
#[cfg(feature = "export")]
use crate::param::{EnvValue, FlagOptionValue, PositionalValue};
use crate::parser::{parse, parse_symbol, Event, EventData, EventScope, Position};
use crate::runtime::Runtime;
use crate::utils::{
    AFTER_HOOK, BEFORE_HOOK, MAIN_NAME, META_AUTHOR, META_COMBINE_SHORTS, META_DEFAULT_SUBCOMMAND,
    META_DOTENV, META_INHERIT_FLAG_OPTIONS, META_REQUIRE_TOOLS, META_SYMBOL, META_VERSION,
    ROOT_NAME,
};
use crate::Result;

use anyhow::{anyhow, bail};
use indexmap::{IndexMap, IndexSet};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Default)]
pub(crate) struct Command {
    pub(crate) name: Option<String>,
    pub(crate) match_fn: Option<String>,
    pub(crate) command_fn: Option<String>,
    pub(crate) paths: Vec<String>,
    pub(crate) describe: String,
    pub(crate) flag_option_params: Vec<FlagOptionParam>,
    pub(crate) derived_flag_option_params: Vec<FlagOptionParam>,
    pub(crate) positional_params: Vec<PositionalParam>,
    pub(crate) env_params: Vec<EnvParam>,
    pub(crate) subcommands: Vec<Command>,
    pub(crate) subcommand_fns: HashMap<String, Position>,
    pub(crate) default_subcommand: Option<(usize, Position)>,
    pub(crate) aliases: Option<(Vec<String>, Position)>,
    pub(crate) author: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) names_checker: NamesChecker,
    pub(crate) share: Arc<RefCell<ShareData>>,
    // (key, value, position)
    pub(crate) metadata: Vec<(String, String, Position)>,
    pub(crate) symbols: IndexMap<char, SymbolParam>,
    pub(crate) require_tools: IndexSet<String>,
    pub(crate) help_flags: Vec<&'static str>,
    pub(crate) version_flags: Vec<&'static str>,
}

impl Command {
    pub(crate) fn new(source: &str, root_name: &str) -> Result<Self> {
        let events = parse(source)?;
        let mut root = Command::new_from_events(&events)?;
        root.share.borrow_mut().name = Some(root_name.to_string());
        root.update_recursively(vec![], IndexSet::new());
        if root.has_metadata(META_INHERIT_FLAG_OPTIONS) {
            root.inherit_flag_options();
        }
        root.inherit_envs();
        Ok(root)
    }

    #[cfg(feature = "eval")]
    pub(crate) fn eval<T: Runtime>(
        &mut self,
        runtime: T,
        args: &[String],
        script_path: Option<&str>,
        wrap_width: Option<usize>,
    ) -> Result<Vec<ArgcValue>> {
        if args.is_empty() {
            bail!("Invalid args");
        }
        if args.len() >= 3 && args[1] == T::INTERNAL_SYMBOL {
            let fallback_args = vec![ROOT_NAME.to_string()];
            let new_args = if args.len() == 3 {
                &fallback_args
            } else {
                &args[3..]
            };
            let matcher = Matcher::new(runtime, self, new_args, false);
            let mut arg_values = matcher.to_arg_values_for_param_fn();
            arg_values.push(ArgcValue::ParamFn(args[2].clone()));
            return Ok(arg_values);
        }
        let mut matcher = Matcher::new(runtime, self, args, false);
        if let Some(script_path) = script_path {
            matcher.set_script_path(script_path)
        }
        if let Some(wrap_width) = wrap_width {
            matcher.set_wrap_width(wrap_width)
        }
        Ok(matcher.to_arg_values())
    }

    #[cfg(feature = "export")]
    pub(crate) fn export(&self) -> CommandValue {
        use serde_json::json;

        let mut extra: IndexMap<String, serde_json::Value> = IndexMap::new();
        let require_tools = self.meta_require_tools();
        if !require_tools.is_empty() {
            extra.insert("require_tools".into(), require_tools.into());
        }
        if self.is_root() {
            if self.get_metadata(META_COMBINE_SHORTS).is_some() {
                extra.insert("combine_shorts".into(), true.into());
            }
            if let Some(dotenv) = self.dotenv() {
                extra.insert("dotenv".into(), dotenv.into());
            }
            let (before_hook, after_hook) = self.exist_hooks();
            if before_hook {
                extra.insert("before_hook".into(), BEFORE_HOOK.into());
            }
            if after_hook {
                extra.insert("after_hook".into(), AFTER_HOOK.into());
            }
        } else if let Some((idx, _)) = &self.default_subcommand {
            extra.insert("default_subcommand".into(), (*idx).into());
        }
        if !self.metadata.is_empty() {
            extra.insert(
                "metadata".into(),
                json!(self
                    .metadata
                    .iter()
                    .map(|(k, v, _)| (
                        k.to_string(),
                        if v.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    ))
                    .collect::<IndexMap<String, Option<String>>>()),
            );
        }
        extra.insert("command_fn".into(), self.command_fn.clone().into());
        let flag_options = self.all_flag_options().iter().map(|v| v.export()).collect();
        CommandValue {
            name: self.cmd_name(),
            describe: self.describe.clone(),
            author: self.author.clone(),
            version: self.version.clone(),
            aliases: self.list_alias_names().clone(),
            flag_options,
            positionals: self.positional_params.iter().map(|v| v.export()).collect(),
            envs: self.env_params.iter().map(|v| v.export()).collect(),
            subcommands: self.subcommands.iter().map(|v| v.export()).collect(),
            extra,
        }
    }

    pub(crate) fn new_from_events(events: &[Event]) -> Result<Self> {
        let mut root_cmd = Command::default();
        let share_data = root_cmd.share.clone();
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
                    match key.as_str() {
                        META_SYMBOL => {
                            let (ch, name, choice_fn) = parse_symbol(&value).ok_or_else(|| {
                                anyhow!("@meta(line {}) invalid symbol value", position)
                            })?;
                            cmd.symbols
                                .insert(ch, (name.to_string(), choice_fn.map(|v| v.to_string())));
                        }
                        META_VERSION => {
                            if value.is_empty() {
                                bail!("@meta(line {}) invalid version value", position)
                            } else {
                                cmd.version = Some(value.clone());
                            }
                        }
                        META_AUTHOR => {
                            if value.is_empty() {
                                bail!("@meta(line {}) invalid version value", position)
                            } else {
                                cmd.author = Some(value.clone());
                            }
                        }
                        _ => {}
                    }
                    cmd.metadata.push((key, value, position));
                }
                EventData::Cmd(value) => {
                    if share_data.borrow().scope == EventScope::CmdStart {
                        bail!(
                            "@cmd(line {}) missing function?",
                            share_data.borrow().cmd_pos
                        )
                    }
                    share_data.borrow_mut().cmd_pos = position;
                    share_data.borrow_mut().scope = EventScope::CmdStart;
                    let subcmd = root_cmd.create_cmd();
                    if !value.is_empty() {
                        subcmd.describe.clone_from(&value);
                    }
                }
                EventData::Aliases(values) => {
                    let cmd = Self::get_cmd(&mut root_cmd, "@alias", position)?;
                    cmd.aliases = Some((values.to_vec(), position));
                }
                EventData::FlagOption(param) => {
                    param.guard().map_err(|err| {
                        anyhow!("{}(line {}) is invalid, {err}", param.tag_name(), position)
                    })?;
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    if param.is_option() {
                        share_data.borrow_mut().add_param_fn(
                            position,
                            param.default_fn(),
                            param.choice_fn(),
                        );
                    }
                    cmd.names_checker.check_flag_option(&param, position)?;
                    cmd.flag_option_params.push(param);
                }
                EventData::Env(param) => {
                    param.guard().map_err(|err| {
                        anyhow!("{}(line {}) is invalid, {err}", param.tag_name(), position)
                    })?;
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    share_data.borrow_mut().add_param_fn(
                        position,
                        param.default_fn(),
                        param.choice_fn(),
                    );
                    cmd.names_checker.check_env(&param, position)?;
                    cmd.env_params.push(param);
                }
                EventData::Positional(param) => {
                    param.guard().map_err(|err| {
                        anyhow!("{}(line {}) is invalid, {err}", param.tag_name(), position)
                    })?;
                    let cmd = Self::get_cmd(&mut root_cmd, param.tag_name(), position)?;
                    share_data.borrow_mut().add_param_fn(
                        position,
                        param.default_fn(),
                        param.choice_fn(),
                    );
                    cmd.add_positional_param(param, position)?;
                }
                EventData::Func(name) => {
                    if let Some(pos) = share_data.borrow_mut().cmd_fns.get(&name) {
                        bail!(
                            "{}(line {}) conflicts with cmd or alias at line {}",
                            name,
                            position,
                            pos
                        )
                    }
                    share_data.borrow_mut().fns.insert(name.clone(), position);
                    if share_data.borrow().scope == EventScope::CmdStart {
                        share_data
                            .borrow_mut()
                            .cmd_fns
                            .insert(name.clone(), position);

                        let parts: Vec<&str> = name.split("::").collect();
                        let parts_len = parts.len();
                        if parts_len == 0 {
                            bail!("{}(line {}) invalid command name", name, position);
                        }
                        if parts_len == 1 {
                            let cmd = root_cmd.subcommands.last_mut().unwrap();
                            cmd.name = Some(sanitize_cmd_name(&name));
                            cmd.match_fn = Some(name.to_string());
                            if let Some((aliases, aliases_pos)) = &cmd.aliases {
                                for name in aliases {
                                    if let Some(pos) = share_data.borrow().cmd_fns.get(name) {
                                        bail!(
                                            "@alias(line {}) conflicts with cmd or alias at line {}",
                                            aliases_pos,
                                            pos
                                        );
                                    }
                                    share_data
                                        .borrow_mut()
                                        .cmd_fns
                                        .insert(name.clone(), *aliases_pos);
                                }
                            }
                            update_parent_cmd(&mut root_cmd)?;
                        } else {
                            let mut cmd = root_cmd.subcommands.pop().unwrap();
                            let (child, parents) = parts.split_last().unwrap();
                            let parents: Vec<String> =
                                parents.iter().map(|v| sanitize_cmd_name(v)).collect();
                            cmd.name = Some(sanitize_cmd_name(child));
                            cmd.match_fn = Some(name.to_string());
                            match retrieve_cmd(&mut root_cmd, &parents) {
                                Some(parent_cmd) => {
                                    parent_cmd
                                        .subcommand_fns
                                        .insert(child.to_string(), position);
                                    if let Some((aliases, aliases_pos)) = &cmd.aliases {
                                        for name in aliases {
                                            if let Some(pos) = parent_cmd.subcommand_fns.get(name) {
                                                bail!(
												"@alias(line {}) conflicts with cmd or alias at line {}",
												aliases_pos,
												pos
											);
                                            }
                                            parent_cmd
                                                .subcommand_fns
                                                .insert(name.clone(), *aliases_pos);
                                        }
                                    }
                                    parent_cmd.subcommands.push(cmd);
                                    update_parent_cmd(parent_cmd)?;
                                }
                                None => {
                                    bail!("{}(line {}) lack of parent command", name, position);
                                }
                            }
                        }
                    }
                    share_data.borrow_mut().scope = EventScope::FnEnd;
                }
                EventData::Unknown(name) => {
                    bail!("@{}(line {}) is unknown tag", name, position);
                }
            }
        }
        root_cmd.share.borrow().check_param_fn()?;
        Ok(root_cmd)
    }

    pub(crate) fn has_metadata(&self, key: &str) -> bool {
        self.metadata.iter().any(|(k, _, _)| k == key)
    }

    pub(crate) fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, v, _)| v.as_str())
    }

    pub(crate) fn meta_require_tools(&self) -> Vec<String> {
        let raw_require_tools = self.get_metadata(META_REQUIRE_TOOLS).unwrap_or_default();
        if raw_require_tools.is_empty() {
            vec![]
        } else {
            raw_require_tools
                .split(',')
                .map(|v| v.to_string())
                .collect()
        }
    }

    pub(crate) fn flag_option_signs(&self) -> String {
        let mut signs: IndexSet<char> = ['-'].into();
        for param in &self.flag_option_params {
            if let Some(short) = param.short() {
                signs.extend(short.chars().take(1))
            }
            signs.extend(param.long_prefix().chars().take(1))
        }
        signs.into_iter().collect()
    }

    pub(crate) fn cmd_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| self.share.borrow().name())
    }

    pub(crate) fn is_root(&self) -> bool {
        self.paths.is_empty()
    }

    pub(crate) fn cmd_paths(&self) -> Vec<String> {
        let root_name = self.share.borrow().name();
        let mut paths = self.paths.clone();
        paths.insert(0, root_name);
        paths
    }

    pub(crate) fn full_name(&self) -> String {
        self.cmd_paths().join("-")
    }

    pub(crate) fn render_version(&self) -> String {
        format!(
            "{} {}",
            self.full_name(),
            self.version.clone().unwrap_or_else(|| "0.0.0".to_string())
        )
    }

    pub(crate) fn describe_oneline(&self) -> &str {
        match self.describe.split_once('\n') {
            Some((v, _)) => v,
            None => self.describe.as_str(),
        }
    }

    pub(crate) fn list_names(&self) -> Vec<String> {
        let mut output: Vec<String> = match self.name.clone() {
            Some(v) => vec![v],
            None => vec![],
        };
        output.extend(self.list_alias_names());
        output
    }

    pub(crate) fn list_alias_names(&self) -> Vec<String> {
        match &self.aliases {
            Some((v, _)) => v.clone(),
            None => vec![],
        }
    }

    pub(crate) fn list_subcommand_names(&self) -> Vec<String> {
        self.subcommands
            .iter()
            .flat_map(|v| v.list_names())
            .collect()
    }

    pub(crate) fn find_subcommand(&self, name: &str) -> Option<&Self> {
        self.subcommands.iter().find(|subcmd| {
            return subcmd.list_names().iter().any(|v| v == name);
        })
    }

    pub(crate) fn find_default_subcommand(&self) -> Option<&Self> {
        let (idx, _) = self.default_subcommand.as_ref()?;
        Some(&self.subcommands[*idx])
    }

    pub(crate) fn find_flag_option(&self, name: &str) -> Option<&FlagOptionParam> {
        self.flag_option_params.iter().find(|v| v.is_match(name))
    }

    pub(crate) fn find_env(&self, name: &str) -> Option<&EnvParam> {
        self.env_params.iter().find(|v| v.id() == name)
    }

    pub(crate) fn all_flag_options(&self) -> Vec<&FlagOptionParam> {
        let mut list: Vec<&FlagOptionParam> = self.flag_option_params.iter().collect();
        list.extend(self.derived_flag_option_params.iter());
        list
    }

    pub(crate) fn is_empty_flags_options_subcommands(&self) -> bool {
        self.flag_option_params.is_empty() && self.subcommands.is_empty()
    }

    pub(crate) fn exist_hooks(&self) -> (bool, bool) {
        let fns = &self.share.borrow().fns;
        let before_hook = fns.contains_key(BEFORE_HOOK);
        let after_hook = fns.contains_key(AFTER_HOOK);
        (before_hook, after_hook)
    }

    pub(crate) fn exist_version(&self) -> bool {
        self.version.is_some() || self.is_root()
    }

    pub(crate) fn delegated(&self) -> bool {
        self.subcommands.is_empty()
            && self.flag_option_params.is_empty()
            && self.positional_params.len() == 1
            && self.positional_params[0].terminated()
    }

    pub(crate) fn dotenv(&self) -> Option<&str> {
        let dotenv = self.get_metadata(META_DOTENV)?;
        let dotenv = if dotenv.is_empty() { ".env" } else { dotenv };
        Some(dotenv)
    }

    fn update_recursively(&mut self, paths: Vec<String>, mut require_tools: IndexSet<String>) {
        self.paths.clone_from(&paths);

        // update command_fn
        if paths.is_empty() {
            if self.share.borrow().fns.contains_key(MAIN_NAME) {
                self.command_fn = Some(MAIN_NAME.to_string())
            }
        } else if self.subcommands.is_empty() {
            self.command_fn.clone_from(&self.match_fn);
        } else {
            let command_fn = [paths.as_slice(), [MAIN_NAME.to_string()].as_slice()]
                .concat()
                .join("::");
            if self.share.borrow().fns.contains_key(&command_fn) {
                self.command_fn = Some(command_fn)
            }
        }

        self.help_flags = {
            let mut flags = vec!["--help", "-help"];
            let short = match self.find_flag_option("-h") {
                Some(param) => param.id() == "help",
                None => true,
            };
            if short {
                flags.push("-h");
            }
            flags
        };
        self.version_flags = {
            let mut flags = vec![];
            if self.exist_version() {
                flags.push("--version");
                flags.push("-version");
                let short = match self.find_flag_option("-V") {
                    Some(param) => param.id() == "version",
                    None => true,
                };
                if short {
                    flags.push("-V");
                }
            }
            flags
        };

        // update derived_flag_option_params
        let mut describe = false;
        let mut single = false;
        for param in self.flag_option_params.iter() {
            if param.long_prefix().len() == 1 {
                single = true;
            }
            if !param.describe().is_empty() {
                describe = true;
            }
        }
        let long_prefix = if single { "-" } else { "--" };
        self.derived_flag_option_params.extend(
            [
                self.create_help_flag(describe, long_prefix),
                self.create_version_flag(describe, long_prefix),
            ]
            .into_iter()
            .flatten(),
        );

        // update require_tools
        require_tools.extend(self.meta_require_tools());
        self.require_tools = require_tools;

        // update recursively
        for subcmd in self.subcommands.iter_mut() {
            let mut parents = paths.clone();
            parents.push(subcmd.name.clone().unwrap_or_default());
            subcmd.update_recursively(parents, self.require_tools.clone());
        }
    }

    fn inherit_flag_options(&mut self) {
        for subcmd in self.subcommands.iter_mut() {
            let mut inherited_flag_options = vec![];
            for flag_option in &self.flag_option_params {
                if subcmd.find_flag_option(flag_option.id()).is_none() {
                    let mut flag_option = flag_option.clone();
                    flag_option.set_inherit();
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
                if subcmd.find_env(env_param.id()).is_none() {
                    let mut env_param = env_param.clone();
                    env_param.set_inherit();
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
        if cmd.share.borrow().scope == EventScope::FnEnd {
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
            share: self.share.clone(),
            names_checker: Default::default(),
            ..Default::default()
        };
        self.subcommands.push(cmd);
        self.subcommands.last_mut().unwrap()
    }

    fn create_help_flag(&self, describe: bool, long_prefix: &str) -> Option<FlagOptionParam> {
        if self.find_flag_option("help").is_some() {
            return None;
        }
        let describe = if describe { "Print help" } else { "" };
        let short = if self.find_flag_option("-h").is_none() {
            Some("-h")
        } else {
            None
        };
        Some(FlagOptionParam::create_help_flag(
            short,
            long_prefix,
            describe,
        ))
    }

    fn create_version_flag(&self, describe: bool, long_prefix: &str) -> Option<FlagOptionParam> {
        if !self.exist_version() {
            return None;
        }
        if self.find_flag_option("version").is_some() {
            return None;
        }
        let describe = if describe { "Print version" } else { "" };
        let short = if self.find_flag_option("-V").is_none() {
            Some("-V")
        } else {
            None
        };
        Some(FlagOptionParam::create_version_flag(
            short,
            long_prefix,
            describe,
        ))
    }
}

#[cfg(any(feature = "build", feature = "eval"))]
impl Command {
    pub(crate) fn render_help(&self, wrap_width: Option<usize>) -> String {
        let mut output = vec![];
        if self.version.is_some() {
            output.push(self.render_version());
        }
        if let Some(author) = &self.author {
            output.push(author.to_string());
        }
        if !&self.describe.is_empty() {
            output.push(render_block("", &self.describe, wrap_width));
        }
        if !output.is_empty() {
            output.push(String::new());
        }
        output.push(self.render_usage());
        output.push(String::new());
        output.extend(self.render_positionals(wrap_width));
        output.extend(self.render_flag_options(wrap_width));
        output.extend(self.render_subcommands(wrap_width));
        output.extend(self.render_envs(wrap_width));
        if output.is_empty() {
            return "\n".to_string();
        }
        output.join("\n")
    }

    fn render_usage(&self) -> String {
        let mut output = vec!["USAGE:".to_string()];
        output.extend(self.cmd_paths());
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
        if !self.subcommands.is_empty() {
            output.push("<COMMAND>".to_string());
        } else {
            output.extend(self.positional_params.iter().map(|v| v.render_notation()));
        }
        output.join(" ")
    }

    fn render_flag_options(&self, wrap_width: Option<usize>) -> Vec<String> {
        let mut output = vec![];
        let default_subcmd = self.find_default_subcommand();
        if self.flag_option_params.is_empty()
            && default_subcmd
                .map(|subcmd| subcmd.flag_option_params.is_empty())
                .unwrap_or(true)
        {
            return output;
        }

        let params = match default_subcmd {
            Some(subcmd) => [self.all_flag_options(), subcmd.all_flag_options()].concat(),
            None => self.all_flag_options(),
        };

        let mut value_size = 0;
        let list: IndexMap<String, String> = params
            .into_iter()
            .map(|param| {
                let value = param.render_body();
                let describe = param.render_describe();
                value_size = value_size.max(value.len());
                (value, describe)
            })
            .collect();
        value_size += 2;
        output.push("OPTIONS:".to_string());
        render_list(
            &mut output,
            list.into_iter().collect(),
            value_size,
            wrap_width,
        );
        output
    }

    fn render_positionals(&self, wrap_width: Option<usize>) -> Vec<String> {
        let mut output = vec![];
        let params = match self.find_default_subcommand() {
            Some(subcmd) => &subcmd.positional_params,
            None => &self.positional_params,
        };
        if params.is_empty() {
            return output;
        }
        let mut value_size = 0;
        let list: Vec<_> = params
            .iter()
            .map(|param| {
                let value = param.render_notation();
                value_size = value_size.max(value.len());
                (value, param.render_describe())
            })
            .collect();
        value_size += 2;
        output.push("ARGS:".to_string());
        render_list(&mut output, list, value_size, wrap_width);
        output
    }

    fn render_envs(&self, wrap_width: Option<usize>) -> Vec<String> {
        let mut output = vec![];
        let params = match self.find_default_subcommand() {
            Some(subcmd) => &subcmd.env_params,
            None => &self.env_params,
        };
        if params.is_empty() {
            return output;
        }
        let mut value_size = 0;
        let list: Vec<_> = params
            .iter()
            .map(|param| {
                let value = param.render_body();
                value_size = value_size.max(value.len());
                (value, param.render_describe())
            })
            .collect();
        value_size += 2;
        output.push("ENVIRONMENTS:".to_string());
        render_list(&mut output, list, value_size, wrap_width);
        output
    }

    fn render_subcommands(&self, wrap_width: Option<usize>) -> Vec<String> {
        let mut output = vec![];
        if self.subcommands.is_empty() {
            return output;
        }
        let mut value_size = 0;
        let list: Vec<_> = self
            .subcommands
            .iter()
            .map(|subcmd| {
                let value = subcmd.cmd_name();
                value_size = value_size.max(value.len());
                (value, subcmd.render_subcommand_describe())
            })
            .collect();
        value_size += 2;
        output.push("COMMANDS:".to_string());
        render_list(&mut output, list, value_size, wrap_width);
        output
    }

    fn render_subcommand_describe(&self) -> String {
        let mut output = self.describe_oneline().to_string();
        if let Some((aliases, _)) = &self.aliases {
            if !output.is_empty() {
                output.push(' ')
            }
            output.push_str(&format!("[aliases: {}]", aliases.join(", ")));
        }
        if self.has_metadata(META_DEFAULT_SUBCOMMAND) {
            if !output.is_empty() {
                output.push(' ')
            }
            output.push_str("[default]");
        }
        output
    }
}

#[cfg(feature = "export")]
#[derive(Debug, Serialize)]
pub struct CommandValue {
    pub name: String,
    pub describe: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub aliases: Vec<String>,
    pub flag_options: Vec<FlagOptionValue>,
    pub positionals: Vec<PositionalValue>,
    pub envs: Vec<EnvValue>,
    pub subcommands: Vec<CommandValue>,
    #[serde(flatten)]
    pub extra: IndexMap<String, serde_json::Value>,
}

pub(crate) type SymbolParam = (String, Option<String>);

fn retrieve_cmd<'a>(cmd: &'a mut Command, paths: &[String]) -> Option<&'a mut Command> {
    if paths.is_empty() {
        return Some(cmd);
    }
    let child = cmd
        .subcommands
        .iter_mut()
        .find(|v| v.name.as_deref() == Some(paths[0].as_str()))?;
    retrieve_cmd(child, &paths[1..])
}

fn update_parent_cmd(parent: &mut Command) -> Result<()> {
    let index = parent.subcommands.len() - 1;
    let subcmd = &parent.subcommands[index];
    if let Some((_, _, meta_pos)) = subcmd
        .metadata
        .iter()
        .find(|(k, _, _)| k == META_DEFAULT_SUBCOMMAND)
    {
        if !parent.positional_params.is_empty() {
            bail!(
                "@meta(line {}) can't be added since the parent command has positional parameters",
                meta_pos
            )
        }
        if let Some((_, exist_pos)) = &parent.default_subcommand {
            bail!("@meta(line {}) conflicts with {}", meta_pos, exist_pos)
        } else {
            parent.default_subcommand = Some((index, *meta_pos))
        }
    }
    Ok(())
}

fn sanitize_cmd_name(name: &str) -> String {
    name.trim_end_matches('_').to_string()
}

fn render_list(
    output: &mut Vec<String>,
    list: Vec<(String, String)>,
    value_size: usize,
    wrap_width: Option<usize>,
) {
    let mut mapped_list = vec![];
    let multiline = list.iter().any(|(_, describe)| describe.contains('\n'));
    for (value, describe) in list {
        let item = if describe.is_empty() {
            let maybe_newline = if multiline { "\n" } else { "" };
            format!("  {value}{maybe_newline}")
        } else if multiline {
            format!(
                "  {value}\n{}\n",
                render_block(&" ".repeat(10), &describe, wrap_width)
            )
        } else {
            let spaces = " ".repeat(value_size - value.len());
            render_block(&format!("  {value}{spaces}"), &describe, wrap_width)
        };
        mapped_list.push(item);
    }
    for item in mapped_list {
        output.push(item);
    }
    if !multiline {
        output.push("".to_string());
    }
}

fn render_block(name: &str, describe: &str, wrap_width: Option<usize>) -> String {
    let size = wrap_width.unwrap_or(999) - name.len();
    let empty = " ".repeat(name.len());
    describe
        .split('\n')
        .flat_map(|v| {
            #[cfg(feature = "wrap-help")]
            {
                textwrap::wrap(v, size)
            }
            #[cfg(not(feature = "wrap-help"))]
            {
                vec![v]
            }
        })
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
