use std::{
    collections::{HashMap, HashSet},
    env,
    path::MAIN_SEPARATOR,
};

use crate::{
    command::Command,
    compgen::CompKind,
    param::{FlagOptionParam, PositionalParam},
    utils::run_param_fns,
    ArgcValue, Shell,
};

use either::Either;
use indexmap::{IndexMap, IndexSet};

const KNOWN_OPTIONS: [&str; 6] = ["-h", "-help", "--help", "-V", "-version", "--version"];

pub(crate) struct Matcher<'a, 'b> {
    cmds: Vec<(&'b str, &'a Command, String, usize)>,
    args: &'b [String],
    flag_option_args: Vec<Vec<FlagOptionArg<'a, 'b>>>,
    positional_args: Vec<&'b str>,
    dashes: Vec<usize>,
    arg_comp: ArgComp,
    choices_fns: HashSet<&'a str>,
    choices_values: HashMap<&'a str, Vec<String>>,
    script_path: Option<String>,
    term_width: Option<usize>,
    is_last_arg_option_assign: bool,
}

type FlagOptionArg<'a, 'b> = (&'b str, Vec<&'b str>, Option<&'a str>);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ArgComp {
    FlagOrOption,
    FlagOrOptionCombine(String),
    CommandOrPositional,
    OptionValue(String, usize),
    Any,
}

impl ArgComp {
    pub(crate) fn is_flag_or_option(&self) -> bool {
        matches!(self, Self::FlagOrOption | &Self::OptionValue(_, _))
    }
}

#[derive(Debug)]
pub(crate) enum MatchError {
    DisplayHelp,
    DisplaySubcommandHelp(String),
    DisplayVersion,
    InvalidSubcommand,
    UnknownArgument(usize, String),
    MissingRequiredArgument(usize, Vec<String>),
    NotMultipleArgument(usize, String),
    InvalidValue(usize, String, String, Vec<String>),
    MismatchValues(usize, String),
    NoMoreValue(usize, String, String),
}

impl<'a, 'b> Matcher<'a, 'b> {
    pub(crate) fn new(root: &'a Command, args: &'b [String]) -> Self {
        let mut cmds = vec![(args[0].as_str(), root, args[0].clone(), 0)];
        let mut cmd_level = 0;
        let mut arg_index = 1;
        let mut flag_option_args = vec![vec![]];
        let mut positional_args = vec![];
        let mut dashes = vec![];
        let mut is_rest_args_positional = false;
        let mut is_last_arg_option_assign = false;
        let mut arg_comp = ArgComp::Any;
        let mut choices_fns = HashSet::new();
        let args_len = args.len();
        if let Some(arg) = args.last() {
            if arg.starts_with('-') {
                arg_comp = ArgComp::FlagOrOption;
            } else if !arg.is_empty() {
                arg_comp = ArgComp::CommandOrPositional;
            }
        }
        while arg_index < args_len {
            let cmd = cmds[cmd_level].1;
            let arg = args[arg_index].as_str();
            if arg == "--" {
                dashes.push(positional_args.len());
                if is_rest_args_positional {
                    add_positional_arg(
                        &mut positional_args,
                        arg,
                        &mut is_rest_args_positional,
                        cmd,
                    );
                }
            } else if is_rest_args_positional
                || !dashes.is_empty()
                || (cmd.no_flags_options_subcommands() && !KNOWN_OPTIONS.contains(&arg))
            {
                add_positional_arg(&mut positional_args, arg, &mut is_rest_args_positional, cmd);
            } else if arg.starts_with('-') {
                if let Some((k, v)) = arg.split_once('=') {
                    let param = cmd.find_flag_option(k);
                    if arg_index == args_len - 1 {
                        if let Some(param) = param {
                            arg_comp = ArgComp::OptionValue(param.name.clone(), 0)
                        }
                        is_last_arg_option_assign = true;
                    }
                    if let Some((choices_fn, validate)) = param.and_then(|v| v.choices_fn.as_ref())
                    {
                        if *validate {
                            choices_fns.insert(choices_fn.as_str());
                        }
                    }
                    flag_option_args[cmd_level].push((k, vec![v], param.map(|v| v.name.as_str())));
                } else if let Some(param) = cmd.find_flag_option(arg) {
                    if let Some((choices_fn, validate)) = param.choices_fn.as_ref() {
                        if *validate {
                            choices_fns.insert(choices_fn.as_str());
                        }
                    }
                    match_flag_option(
                        &mut flag_option_args[cmd_level],
                        args,
                        &mut arg_index,
                        param,
                        &mut arg_comp,
                    );
                } else if let Some(mut list) = match_combine_shorts(cmd, arg) {
                    let name = list.pop().and_then(|v| v.2).unwrap();
                    let param = cmd.find_flag_option(name).unwrap();
                    if let Some((choices_fn, validate)) = param.choices_fn.as_ref() {
                        if *validate {
                            choices_fns.insert(choices_fn.as_str());
                        }
                    }
                    flag_option_args[cmd_level].extend(list);
                    match_flag_option(
                        &mut flag_option_args[cmd_level],
                        args,
                        &mut arg_index,
                        param,
                        &mut arg_comp,
                    );
                } else {
                    flag_option_args[cmd_level].push((arg, vec![], None));
                }
            } else if let Some(subcmd) = cmd.find_subcommand(arg) {
                cmd_level += 1;
                cmds.push((
                    arg,
                    subcmd,
                    subcmd.name.clone().unwrap_or_else(|| arg.to_string()),
                    arg_index,
                ));
                flag_option_args.push(vec![]);
            } else {
                add_positional_arg(&mut positional_args, arg, &mut is_rest_args_positional, cmd);
            }
            arg_index += 1;
        }

        if is_rest_args_positional {
            arg_comp = ArgComp::CommandOrPositional;
        }

        let last_cmd = cmds.last().unwrap().1;
        choices_fns.extend(last_cmd.positional_params.iter().filter_map(|v| {
            if let Some((choices_fn, validate)) = v.choices_fn.as_ref() {
                if *validate {
                    return Some(choices_fn.as_str());
                }
            }
            None
        }));
        Self {
            cmds,
            args,
            flag_option_args,
            positional_args,
            dashes,
            arg_comp,
            choices_fns,
            choices_values: HashMap::new(),
            script_path: None,
            term_width: None,
            is_last_arg_option_assign,
        }
    }

    pub(crate) fn set_script_path(&mut self, script_path: &str) {
        self.script_path = Some(script_path.to_string());
        let fns: Vec<&str> = self.choices_fns.iter().copied().collect();
        let mut envs = HashMap::new();
        envs.insert("ARGC_OS".into(), env::consts::OS.to_string());
        envs.insert("ARGC_PATH_SEP".into(), MAIN_SEPARATOR.into());
        if let Some(outputs) = run_param_fns(script_path, &fns, self.args, envs) {
            for (i, output) in outputs.into_iter().enumerate() {
                let choices = output
                    .split('\n')
                    .filter_map(|v| {
                        let v = v.trim_matches(|c: char| c.is_whitespace() || c == '\0');
                        if v.is_empty() {
                            None
                        } else {
                            Some(v.to_string())
                        }
                    })
                    .collect();
                self.choices_values.insert(fns[i], choices);
            }
        }
    }

    pub(crate) fn set_term_width(&mut self, term_width: usize) {
        self.term_width = Some(term_width);
    }

    pub(crate) fn to_arg_values(&self) -> Vec<ArgcValue> {
        if let Some(err) = self.validate() {
            return vec![ArgcValue::Error(self.stringify_match_error(&err))];
        }
        let (cmd, cmd_paths) = self.get_cmd_and_paths(self.cmds.len() - 1);
        let mut output = self.to_arg_values_base();
        if cmd.positional_params.is_empty() && !self.positional_args.is_empty() {
            output.push(ArgcValue::ExtraPositionalMultiple(
                self.positional_args.iter().map(|v| v.to_string()).collect(),
            ));
        }
        if let Some(cmd_fn) = cmd.get_cmd_fn(&cmd_paths) {
            output.push(ArgcValue::CmdFn(cmd_fn));
        }
        output
    }

    pub(crate) fn to_arg_values_for_choice_fn(&self) -> Vec<ArgcValue> {
        let mut output: Vec<ArgcValue> = self.to_arg_values_base();
        if !self.dashes.is_empty() {
            output.push(ArgcValue::Multiple(
                "_dashes".into(),
                self.dashes.iter().map(|v| v.to_string()).collect(),
            ));
        }
        output
    }

    pub(crate) fn compgen(&self, shell: Shell) -> Vec<CompItem> {
        let redirect_symbols = shell.redirect_symbols();
        if self
            .args
            .iter()
            .any(|v| redirect_symbols.contains(&v.as_str()))
        {
            return vec![("__argc_value=file".into(), String::new(), CompKind::Value)];
        }
        let level = self.cmds.len() - 1;
        let mut last_cmd = self.cmds[level].1;
        let mut output = match &self.arg_comp {
            ArgComp::FlagOrOption => {
                let mut output = self.comp_flag_options();
                if let Some((value, param)) = self
                    .args
                    .last()
                    .and_then(|value| last_cmd.find_flag_option(value).map(|param| (value, param)))
                {
                    let describe = param.describe_head();
                    let kind = if param.is_flag() {
                        CompKind::Flag
                    } else {
                        CompKind::Option
                    };
                    output.push((value.clone(), describe.into(), kind));
                }
                output
            }
            ArgComp::FlagOrOptionCombine(value) => {
                let mut output: Vec<CompItem> = self
                    .comp_flag_options()
                    .iter()
                    .filter_map(|(x, y, z)| {
                        if x.len() == 2 {
                            Some((format!("{value}{}", &x[1..]), y.to_string(), *z))
                        } else {
                            None
                        }
                    })
                    .collect();
                if output.len() == 1 {
                    output.insert(0, (value.to_string(), String::new(), CompKind::Flag));
                }
                output
            }
            ArgComp::CommandOrPositional => {
                if self.positional_args.len() == 2 && self.positional_args[0] == "help" {
                    return comp_subcomands(last_cmd);
                }
                if level > 0
                    && self.positional_args.is_empty()
                    && self.flag_option_args[level].is_empty()
                {
                    last_cmd = self.cmds[level - 1].1;
                }
                let values = self.match_positionals();
                comp_subcommands_positional(last_cmd, &values, self.positional_args.len() < 2)
            }
            ArgComp::OptionValue(name, index) => {
                if let Some(param) = last_cmd.flag_option_params.iter().find(|v| &v.name == name) {
                    comp_flag_option(param, *index)
                } else {
                    vec![]
                }
            }
            ArgComp::Any => {
                if self.positional_args.len() == 2 && self.positional_args[0] == "help" {
                    return comp_subcomands(last_cmd);
                }
                let values = self.match_positionals();
                comp_subcommands_positional(last_cmd, &values, self.positional_args.len() < 2)
            }
        };
        if output.is_empty()
            && !self.arg_comp.is_flag_or_option()
            && last_cmd.positional_params.is_empty()
        {
            output.push(("__argc_value=file".into(), String::new(), CompKind::Value));
        }
        output
    }

    pub(crate) fn is_last_arg_option_assign(&self) -> bool {
        self.is_last_arg_option_assign
    }

    fn to_arg_values_base(&self) -> Vec<ArgcValue> {
        let mut output = vec![];
        let cmds_len = self.cmds.len();
        let level = cmds_len - 1;
        let last_cmd = self.cmds[level].1;
        let args_index = self.cmds[level].3;
        for level in 0..cmds_len {
            let args = &self.flag_option_args[level];
            let cmd = self.cmds[level].1;
            for param in cmd.flag_option_params.iter() {
                let values: Vec<&[&str]> = args
                    .iter()
                    .filter_map(|(_, value, name)| {
                        if let Some(true) = name.map(|v| param.name == v) {
                            Some(value.as_slice())
                        } else {
                            None
                        }
                    })
                    .collect();
                if let Some(value) = param.get_arg_value(&values) {
                    output.push(value);
                }
            }
        }

        let positional_values = self.match_positionals();
        for (i, param) in last_cmd.positional_params.iter().enumerate() {
            let values = positional_values
                .get(i)
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if let Some(value) = param.get_arg_value(values) {
                output.push(value);
            }
        }
        output.push(ArgcValue::Multiple("_args".into(), self.args.to_vec()));
        output.push(ArgcValue::Single("_index".into(), args_index.to_string()));
        output
    }

    fn validate(&self) -> Option<MatchError> {
        let cmds_len = self.cmds.len();
        let level = cmds_len - 1;
        let (last_cmd, cmd_paths) = self.get_cmd_and_paths(level);
        let last_args = &self.flag_option_args[level];
        for (key, _, name) in last_args {
            match name {
                Some("help") => return Some(MatchError::DisplayHelp),
                Some("version") => return Some(MatchError::DisplayVersion),
                None => {
                    if *key == "--help"
                        || *key == "-help"
                        || (last_cmd.match_help_short_name() && *key == "-h")
                    {
                        return Some(MatchError::DisplayHelp);
                    } else if *key == "--version"
                        || *key == "-version"
                        || (last_cmd.match_version_short_name() && *key == "-V")
                    {
                        return Some(MatchError::DisplayVersion);
                    }
                }
                _ => {}
            }
        }
        if let Some(&"help") = self.positional_args.first() {
            if self.positional_args.len() < 2 {
                return Some(MatchError::DisplayHelp);
            }
            let name = self.positional_args[1];
            if let Some(subcmd) = last_cmd.find_subcommand(name) {
                return Some(MatchError::DisplaySubcommandHelp(
                    subcmd.name.clone().unwrap(),
                ));
            } else {
                return Some(MatchError::InvalidValue(
                    level,
                    name.into(),
                    "<command>".into(),
                    last_cmd.list_subcommand_names(),
                ));
            }
        }
        if !last_cmd.subcommands.is_empty() {
            if !last_cmd.exist_main_fn(&cmd_paths) {
                if self.positional_args.is_empty() && last_args.is_empty() {
                    return Some(MatchError::DisplayHelp);
                } else {
                    return Some(MatchError::InvalidSubcommand);
                }
            } else if last_cmd.positional_params.is_empty() && !self.positional_args.is_empty() {
                return Some(MatchError::InvalidSubcommand);
            }
        }

        let positional_values = self.match_positionals();
        let positional_values_len = positional_values.len();
        let positional_params_len = last_cmd.positional_params.len();
        if positional_params_len > 0 && positional_values_len > positional_params_len {
            let extra_args = &positional_values[positional_params_len];
            return Some(MatchError::UnknownArgument(
                level,
                extra_args[0].to_string(),
            ));
        }
        let mut missing_level = level;
        let mut missing_params: Vec<String> = if positional_params_len > positional_values_len {
            last_cmd.positional_params[positional_values_len..]
                .iter()
                .filter(|param| param.required)
                .map(|v| v.render_value())
                .collect()
        } else {
            vec![]
        };
        for (i, param) in last_cmd.positional_params.iter().enumerate() {
            if let (Some(values), Some(choices)) = (
                positional_values.get(i),
                get_param_choices(&param.choices, &param.choices_fn, &self.choices_values),
            ) {
                for value in values.iter() {
                    if !choices.contains(&value.to_string()) {
                        return Some(MatchError::InvalidValue(
                            level,
                            value.to_string(),
                            param.render_value(),
                            choices.clone(),
                        ));
                    }
                }
            }
        }
        for level in (0..cmds_len).rev() {
            let args = &self.flag_option_args[level];
            let cmd = self.cmds[level].1;
            let mut flag_option_map = IndexMap::new();
            let mut missing_flag_options: IndexSet<&str> = cmd
                .flag_option_params
                .iter()
                .filter(|v| v.required)
                .map(|v| v.name.as_str())
                .collect();
            for (i, (key, _, name)) in args.iter().enumerate() {
                match *name {
                    Some(name) => {
                        missing_flag_options.remove(name);
                        flag_option_map.entry(name).or_insert(vec![]).push(i);
                    }
                    None => return Some(MatchError::UnknownArgument(level, key.to_string())),
                }
            }
            if !missing_flag_options.is_empty() {
                let missing_flag_options: Vec<String> = missing_flag_options
                    .iter()
                    .filter_map(|v| cmd.find_flag_option(v).map(|v| v.render_name_values()))
                    .collect();
                missing_params.extend(missing_flag_options)
            }
            for (name, indexes) in flag_option_map {
                if let Some(param) = cmd.flag_option_params.iter().find(|v| v.name == name) {
                    let values_list: Vec<&[&str]> =
                        indexes.iter().map(|v| args[*v].1.as_slice()).collect();
                    if !param.multiple && values_list.len() > 1 {
                        return Some(MatchError::NotMultipleArgument(level, param.render_name()));
                    }
                    for values in values_list.iter() {
                        if values.len() != param.values_size() {
                            if param.is_flag() {
                                return Some(MatchError::NoMoreValue(
                                    level,
                                    param.render_name(),
                                    values[0].to_string(),
                                ));
                            } else if !param.multiple {
                                return Some(MatchError::MismatchValues(
                                    level,
                                    param.render_name_values(),
                                ));
                            }
                        }
                        if let Some(choices) = get_param_choices(
                            &param.choices,
                            &param.choices_fn,
                            &self.choices_values,
                        ) {
                            for value in values.iter() {
                                if !choices.contains(&value.to_string()) {
                                    return Some(MatchError::InvalidValue(
                                        level,
                                        value.to_string(),
                                        param.render_single_value(),
                                        choices.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            if !missing_params.is_empty() {
                missing_level = level;
                break;
            }
        }
        if !missing_params.is_empty() {
            return Some(MatchError::MissingRequiredArgument(
                missing_level,
                missing_params,
            ));
        }
        None
    }

    fn match_positionals(&self) -> Vec<Vec<&str>> {
        let mut output = vec![];
        let args_len = self.positional_args.len();
        if args_len == 0 {
            return output;
        }
        let cmd = self.cmds[self.cmds.len() - 1].1;
        let params_len = cmd.positional_params.len();
        let mut arg_index = 0;
        let mut param_index = 0;
        while param_index < params_len && arg_index < args_len {
            let param = &cmd.positional_params[param_index];
            if param.multiple {
                let dash_idx = self.dashes.first().cloned().unwrap_or_default();
                let takes = if param_index == 0
                    && dash_idx > 0
                    && params_len == 2
                    && cmd.positional_params[1].multiple
                {
                    dash_idx
                } else {
                    (args_len - arg_index).saturating_sub(params_len - param_index) + 1
                };
                output.push(self.positional_args[arg_index..(arg_index + takes)].to_vec());
                arg_index += takes;
            } else {
                let arg = self.positional_args[arg_index];
                output.push(vec![arg]);
                arg_index += 1;
            }
            param_index += 1;
        }
        if arg_index < args_len {
            output.push(self.positional_args[arg_index..].to_vec())
        }
        output
    }

    fn stringify_match_error(&self, err: &MatchError) -> (String, i32) {
        let mut exit = 0;
        let footer = "For more information, try '--help'.";
        let message = match err {
            MatchError::DisplayHelp => {
                let (cmd, cmd_paths) = self.get_cmd_and_paths(self.cmds.len() - 1);
                cmd.render_help(&cmd_paths, self.term_width)
            }
            MatchError::DisplaySubcommandHelp(name) => {
                let (cmd, mut cmd_paths) = self.get_cmd_and_paths(self.cmds.len() - 1);
                let cmd = cmd.find_subcommand(name).unwrap();
                cmd_paths.push(name.as_str());
                cmd.render_help(&cmd_paths, self.term_width)
            }
            MatchError::DisplayVersion => {
                let (cmd, cmd_paths) = self.get_cmd_and_paths(self.cmds.len() - 1);
                cmd.render_version(&cmd_paths)
            }
            MatchError::InvalidSubcommand => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(self.cmds.len() - 1);
                let cmd_str = cmd_paths.join("-");
                let usage = cmd.render_usage(&cmd_paths);
                let names = cmd.list_subcommand_names().join(", ");
                format!(
                    r###"error: `{cmd_str}` requires a subcommand but one was not provided
  [subcommands: {names}]

{usage}

{footer}
"###
                )
            }
            MatchError::UnknownArgument(level, name) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                format!(
                    r###"error: unexpected argument `{name}` found

{usage}

{footer}
"###
                )
            }
            MatchError::MissingRequiredArgument(level, values) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                let list = values
                    .iter()
                    .map(|v| format!("  {v}"))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    r###"error: the following required arguments were not provided:
{list}

{usage}

{footer}
"###
                )
            }
            MatchError::NotMultipleArgument(level, name) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                format!(
                    r###"error: the argument `{name}` cannot be used multiple times

{usage}

{footer}
"###
                )
            }
            MatchError::InvalidValue(_level, value, name, choices) => {
                exit = 1;
                let list = choices.join(", ");
                format!(
                    r###"error: invalid value `{value}` for `{name}`
  [possible values: {list}]

{footer}
"###
                )
            }
            MatchError::MismatchValues(level, value) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                format!(
                    r###"error: invalid values for `{value}`

{usage}

{footer}
"###
                )
            }
            MatchError::NoMoreValue(level, name, value) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                format!(
                    r###"error: unexpected value `{value}` for `{name}` found; no more were expected 

{usage}

{footer}
"###
                )
            }
        };
        (message, exit)
    }

    fn get_cmd_and_paths(&self, level: usize) -> (&Command, Vec<&str>) {
        let cmd = self.cmds[level].1;
        let cmd_paths: Vec<&str> = self
            .cmds
            .iter()
            .take(level + 1)
            .map(|v| v.2.as_str())
            .collect();
        (cmd, cmd_paths)
    }

    fn comp_flag_options(&self) -> Vec<CompItem> {
        let mut output = vec![];
        let level = self.cmds.len() - 1;
        let cmd = self.cmds[level].1;
        let args: HashSet<&str> = self.flag_option_args[level]
            .iter()
            .filter_map(|v| v.2)
            .collect();
        for param in cmd.flag_option_params.iter() {
            let exist = args.contains(param.name.as_str());
            if !exist || param.multiple {
                let describe = param.describe_head();
                let kind = if param.is_flag() {
                    CompKind::Flag
                } else {
                    CompKind::Option
                };
                for v in param.list_names() {
                    output.push((v, describe.to_string(), kind))
                }
            }
        }
        output
    }
}

pub(crate) type CompItem = (String, String, CompKind);

fn add_positional_arg<'a>(
    positional_args: &mut Vec<&'a str>,
    arg: &'a str,
    is_rest_args_positional: &mut bool,
    cmd: &Command,
) {
    positional_args.push(arg);
    if !*is_rest_args_positional
        && cmd.positional_params.last().map(|v| v.terminated) == Some(true)
        && positional_args.len() >= cmd.positional_params.len() - 1
    {
        *is_rest_args_positional = true;
    }
}

fn take_value_args(args: &[String], start: usize, len: usize) -> Vec<&str> {
    let mut output = vec![];
    if len == 0 {
        return output;
    }
    let end = (start + len).min(args.len());
    for arg in args.iter().take(end).skip(start) {
        if arg.starts_with('-') {
            break;
        }
        output.push(arg.as_str());
    }
    output
}

fn match_combine_shorts<'a, 'b>(
    cmd: &'a Command,
    arg: &'b str,
) -> Option<Vec<FlagOptionArg<'a, 'b>>> {
    if arg.len() > 2 && !arg.starts_with("--") {
        let mut output = vec![];
        for ch in arg.chars().skip(1) {
            let name: String = format!("-{ch}");
            if let Some(param) = cmd.find_flag_option(&name) {
                output.push((arg, vec![], Some(param.name.as_str())))
            } else {
                return None;
            }
        }
        Some(output)
    } else {
        None
    }
}

fn match_flag_option<'a, 'b>(
    output: &mut Vec<FlagOptionArg<'a, 'b>>,
    args: &'b [String],
    arg_index: &mut usize,
    param: &'a FlagOptionParam,
    arg_comp: &mut ArgComp,
) {
    if param.terminated {
        let value_args: Vec<&str> = args[*arg_index + 1..].iter().map(|v| v.as_str()).collect();
        let arg = &args[*arg_index];
        *arg_index += value_args.len();
        if !value_args.is_empty() {
            *arg_comp =
                ArgComp::OptionValue(param.name.clone(), value_args.len().saturating_sub(1));
        }
        output.push((arg, value_args, Some(param.name.as_str())));
    } else {
        let values_len = param.values_size();
        let args_len = args.len();
        let value_args = take_value_args(args, *arg_index + 1, values_len);
        let arg = &args[*arg_index];
        *arg_index += value_args.len();
        if *arg_index == args_len - 1 {
            if *arg_comp != ArgComp::FlagOrOption
                && param.is_option()
                && value_args.len() <= values_len
            {
                *arg_comp =
                    ArgComp::OptionValue(param.name.clone(), value_args.len().saturating_sub(1));
            } else if *arg_comp == ArgComp::FlagOrOption
                && param.is_flag()
                && !(arg.len() > 2 && param.is_match(arg))
            {
                *arg_comp = ArgComp::FlagOrOptionCombine(arg.to_string());
            }
        }
        output.push((arg, value_args, Some(param.name.as_str())));
    }
}

fn comp_subcommands_positional(
    cmd: &Command,
    values: &[Vec<&str>],
    with_subcmd: bool,
) -> Vec<CompItem> {
    let mut output = vec![];
    if with_subcmd {
        output.extend(comp_subcomands(cmd))
    }
    if values.is_empty() || values.len() > cmd.positional_params.len() {
        return output;
    }
    output.extend(comp_positional(&cmd.positional_params[values.len() - 1]));
    output
}

fn comp_subcomands(cmd: &Command) -> Vec<CompItem> {
    let mut output = vec![];
    for subcmd in cmd.subcommands.iter() {
        let describe = subcmd.describe_head();
        for v in subcmd.list_names() {
            output.push((v, describe.to_string(), CompKind::Command))
        }
    }
    output
}

fn comp_flag_option(param: &FlagOptionParam, index: usize) -> Vec<CompItem> {
    let value_name = param
        .arg_value_names
        .get(index)
        .map(|v| v.as_str())
        .unwrap_or_else(|| param.arg_value_names.last().unwrap());
    comp_param(
        param.describe_head(),
        value_name,
        &param.choices,
        &param.choices_fn,
        &param.multi_char,
    )
}

fn comp_positional(param: &PositionalParam) -> Vec<CompItem> {
    comp_param(
        param.describe_head(),
        &param.arg_value_name,
        &param.choices,
        &param.choices_fn,
        &param.multi_char,
    )
}

fn comp_param(
    describe: &str,
    value_name: &str,
    choices: &Option<Vec<String>>,
    choices_fn: &Option<(String, bool)>,
    multi_char: &Option<char>,
) -> Vec<CompItem> {
    let choices: Option<Either<Vec<String>, String>> = if let Some(choices_fn) = choices_fn {
        Some(Either::Right(choices_fn.0.to_string()))
    } else {
        choices
            .as_ref()
            .map(|choices| Either::Left(choices.iter().map(|v| v.to_string()).collect()))
    };
    let mut output = if let Some(choices) = choices {
        match choices {
            Either::Left(choices) => choices
                .iter()
                .map(|v| (v.to_string(), String::new(), CompKind::Value))
                .collect(),
            Either::Right(choices_fn) => vec![(
                format!("__argc_fn={}", choices_fn),
                String::new(),
                CompKind::Value,
            )],
        }
    } else {
        let value = format!("__argc_value={}", value_name);
        vec![(value, describe.into(), CompKind::Value)]
    };
    if let Some(ch) = multi_char {
        output.insert(
            0,
            (
                format!("__argc_multi={}", ch),
                String::new(),
                CompKind::Value,
            ),
        );
    }
    output
}

fn get_param_choices<'a, 'b: 'a>(
    choices: &'a Option<Vec<String>>,
    choices_fn: &'a Option<(String, bool)>,
    choices_values: &'a HashMap<&str, Vec<String>>,
) -> Option<&'a Vec<String>> {
    choices.as_ref().or_else(|| {
        choices_fn.as_ref().and_then(|(fn_name, validate)| {
            if *validate {
                choices_values.get(fn_name.as_str())
            } else {
                None
            }
        })
    })
}
