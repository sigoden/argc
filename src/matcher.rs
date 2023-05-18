use std::collections::{HashMap, HashSet};

use crate::{
    command::Command,
    param::{FlagOptionParam, PositionalParam},
    utils::run_param_fns,
    ArgcValue,
};

use either::Either;
use indexmap::{IndexMap, IndexSet};

pub struct Matcher<'a, 'b> {
    cmds: Vec<(&'b str, &'a Command, String)>,
    flag_option_args: Vec<Vec<FlagOptionArg<'a, 'b>>>,
    positional_args: Vec<&'b str>,
    dashdash: Option<usize>,
    arg_comp: ArgComp,
    choices_fns: HashSet<&'a str>,
    choices_values: HashMap<&'a str, Vec<String>>,
    script_path: Option<String>,
    term_width: Option<usize>,
}

type FlagOptionArg<'a, 'b> = (&'b str, Vec<&'b str>, Option<&'a str>);

#[derive(Debug, PartialEq, Eq)]
pub enum ArgComp {
    FlagOrOption,
    FlagOrOptionCombine(String),
    CommandOrPositional,
    OptionValue(String, usize),
    Any,
}

#[derive(Debug)]
pub enum MatchError {
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
    pub fn new(root: &'a Command, args: &'b [String]) -> Self {
        let mut cmds = vec![(args[0].as_str(), root, args[0].clone())];
        let mut cmd_level = 0;
        let mut arg_index = 1;
        let mut flag_option_args = vec![vec![]];
        let mut positional_args = vec![];
        let mut dashdash = None;
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
            if dashdash.is_some()
                || (cmd.without_params_or_subcommands()
                    && !["-h", "-help", "--help"].contains(&arg))
            {
                positional_args.push(arg);
            } else if arg.starts_with('-') {
                if arg == "--" {
                    dashdash = Some(positional_args.len());
                } else if let Some((k, v)) = arg.split_once('=') {
                    let param = cmd.find_flag_option(k);
                    if arg_index == args_len - 1 {
                        if let Some(param) = param {
                            arg_comp = ArgComp::OptionValue(param.name.clone(), 0)
                        }
                    }
                    if let Some(choices_fn) = param.and_then(|v| v.choices_fn.as_ref()) {
                        choices_fns.insert(choices_fn.as_str());
                    }
                    flag_option_args[cmd_level].push((k, vec![v], param.map(|v| v.name.as_str())));
                } else if let Some(param) = cmd.find_flag_option(arg) {
                    if let Some(choices_fn) = param.choices_fn.as_ref() {
                        choices_fns.insert(choices_fn.as_str());
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
                    if let Some(choices_fn) = param.choices_fn.as_ref() {
                        choices_fns.insert(choices_fn.as_str());
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
                ));
                flag_option_args.push(vec![]);
            } else {
                positional_args.push(arg);
            }
            arg_index += 1;
        }
        let last_cmd = cmds.last().unwrap().1;
        choices_fns.extend(
            last_cmd
                .positional_params
                .iter()
                .filter_map(|v| v.choices_fn.as_deref()),
        );
        Self {
            cmds,
            flag_option_args,
            positional_args,
            dashdash,
            arg_comp,
            choices_fns,
            choices_values: HashMap::new(),
            script_path: None,
            term_width: None,
        }
    }

    pub fn set_script_path(&mut self, script_path: &str) {
        self.script_path = Some(script_path.to_string());
        let fns: Vec<&str> = self.choices_fns.iter().copied().collect();
        if let Some(list) = run_param_fns(script_path, &fns, "") {
            for (i, fn_output) in list.into_iter().enumerate() {
                let choices: Vec<String> = fn_output
                    .split('\n')
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
                    .collect();
                if !choices.is_empty() {
                    self.choices_values.insert(fns[i], choices);
                }
            }
        }
    }

    pub fn set_term_width(&mut self, term_width: usize) {
        self.term_width = Some(term_width);
    }

    pub fn to_arg_values(&self) -> Vec<ArgcValue> {
        if let Some(err) = self.validate() {
            return vec![ArgcValue::Error(self.stringify_match_error(&err))];
        }
        let (cmd, cmd_paths) = self.get_cmd_and_paths(self.cmds.len() - 1);
        let mut output = if cmd.without_params_or_subcommands() && !self.positional_args.is_empty()
        {
            vec![ArgcValue::PositionalMultiple(
                "_args".into(),
                self.positional_args.iter().map(|v| v.to_string()).collect(),
            )]
        } else {
            self.to_arg_values_base()
        };
        if let Some(cmd_fn) = cmd.get_cmd_fn(&cmd_paths) {
            output.push(ArgcValue::CmdFn(cmd_fn));
        }
        output
    }

    pub fn to_arg_values_for_choice_fn(&self) -> Vec<ArgcValue> {
        let mut output: Vec<ArgcValue> = self.to_arg_values_base();
        if let Some(v) = self.dashdash {
            output.push(ArgcValue::Single("_dashdash".into(), v.to_string()));
        }
        output.push(ArgcValue::Multiple(
            "_args".into(),
            self.positional_args.iter().map(|v| v.to_string()).collect(),
        ));
        output
    }

    pub fn compgen(&self) -> Vec<(String, String)> {
        match &self.arg_comp {
            ArgComp::FlagOrOption => self.comp_flag_options(),
            ArgComp::FlagOrOptionCombine(value) => self
                .comp_flag_options()
                .iter()
                .filter_map(|(x, y)| {
                    if x.len() == 2 {
                        Some((format!("{value}{}", &x[1..]), y.to_string()))
                    } else {
                        None
                    }
                })
                .collect(),
            ArgComp::CommandOrPositional => {
                let level = self.cmds.len() - 1;
                let mut cmd = self.cmds[level].1;
                if self.positional_args.len() == 2 && self.positional_args[0] == "help" {
                    return comp_subcomands(cmd);
                }
                if level > 0
                    && self.positional_args.is_empty()
                    && self.flag_option_args[level].is_empty()
                {
                    cmd = self.cmds[level - 1].1;
                }
                let values = self.match_positionals();
                comp_subcommands_positional(cmd, &values)
            }
            ArgComp::OptionValue(name, index) => {
                let cmd = self.cmds[self.cmds.len() - 1].1;
                if let Some(param) = cmd.flag_option_params.iter().find(|v| &v.name == name) {
                    comp_flag_option(param, *index)
                } else {
                    vec![]
                }
            }
            ArgComp::Any => {
                let cmd = self.cmds[self.cmds.len() - 1].1;
                let mut output = self.comp_flag_options();
                let values = self.match_positionals();
                output.extend(comp_subcommands_positional(cmd, &values));
                if output.is_empty() {
                    output.push(("__argc_value:file".into(), String::new()))
                }
                output
            }
        }
    }

    fn to_arg_values_base(&self) -> Vec<ArgcValue> {
        let mut output = vec![];
        let cmds_len = self.cmds.len();
        let level = cmds_len - 1;
        let last_cmd = self.cmds[level].1;
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
            if self.positional_args.is_empty() && last_args.is_empty() {
                if !last_cmd.exist_main_fn(&cmd_paths) {
                    return Some(MatchError::DisplayHelp);
                }
            } else {
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
        let missing_positionals: Vec<String> = if positional_params_len > positional_values_len {
            last_cmd.positional_params[positional_values_len..]
                .iter()
                .filter(|param| param.required)
                .map(|v| v.render_value())
                .collect()
        } else {
            vec![]
        };
        for (i, param) in last_cmd.positional_params.iter().enumerate() {
            if let (Some(value), Some(choices)) = (
                positional_values.get(i).and_then(|v| v.first()),
                get_param_choices(&param.choices, &param.choices_fn, &self.choices_values),
            ) {
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
        for level in 0..cmds_len {
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
                return Some(MatchError::MissingRequiredArgument(
                    level,
                    [missing_positionals, missing_flag_options].concat(),
                ));
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
                            let value = values[0];
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
        if !missing_positionals.is_empty() {
            return Some(MatchError::MissingRequiredArgument(
                level,
                missing_positionals,
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
                let takes = (args_len - arg_index).saturating_sub(params_len - param_index) + 1;
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
                let output = cmd.render_version(&cmd_paths);
                format!("{output}\n")
            }
            MatchError::InvalidSubcommand => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(self.cmds.len() - 1);
                let cmd_str = cmd_paths.join("-");
                let usage = cmd.render_usage(&cmd_paths);
                let names = cmd.list_subcommand_names().join(", ");
                format!(
                    r###"error: '{cmd_str}' requires a subcommand but one was not provided
  [subcommands: {names}]

{usage}

For more information, try '--help'.
"###
                )
            }
            MatchError::UnknownArgument(level, name) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                format!(
                    r###"error: unexpected argument '{name}' found

{usage}

For more information, try '--help'.
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

For more information, try '--help'.
"###
                )
            }
            MatchError::NotMultipleArgument(level, name) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                format!(
                    r###"error: the argument '{name}' cannot be used multiple times

{usage}

For more information, try '--help'.
"###
                )
            }
            MatchError::InvalidValue(_level, value, name, choices) => {
                exit = 1;
                let list = choices.join(", ");
                format!(
                    r###"error: invalid value '{value}' for '{name}'
  [possible values: {list}]

For more information, try '--help'.
"###
                )
            }
            MatchError::MismatchValues(level, value) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                format!(
                    r###"error: invalid values for '{value}'

{usage}

For more information, try '--help'.
"###
                )
            }
            MatchError::NoMoreValue(level, name, value) => {
                exit = 1;
                let (cmd, cmd_paths) = self.get_cmd_and_paths(*level);
                let usage = cmd.render_usage(&cmd_paths);
                format!(
                    r###"error: unexpected value '{value}' for '{name}' found; no more were expected 

{usage}

For more information, try '--help'.
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

    fn comp_flag_options(&self) -> Vec<(String, String)> {
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
                let describe = param.describe.clone();
                for v in param.list_names() {
                    output.push((v, describe.clone()))
                }
            }
        }
        output
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
    let values_len = param.values_size();
    let args_len = args.len();
    let value_args = take_value_args(args, *arg_index + 1, values_len);
    let arg = &args[*arg_index];
    *arg_index += value_args.len();
    if *arg_index == args_len - 1 {
        if *arg_comp != ArgComp::FlagOrOption && param.is_option() && value_args.len() <= values_len
        {
            *arg_comp =
                ArgComp::OptionValue(param.name.clone(), value_args.len().saturating_sub(1));
        } else if *arg_comp == ArgComp::FlagOrOption && param.is_flag() && !arg.starts_with("--") {
            *arg_comp = ArgComp::FlagOrOptionCombine(arg.to_string());
        }
    }
    output.push((arg, value_args, Some(param.name.as_str())));
}

fn comp_subcommands_positional(cmd: &Command, values: &[Vec<&str>]) -> Vec<(String, String)> {
    let mut output = vec![];
    if values.len() < 2 {
        output.extend(comp_subcomands(cmd))
    }
    if values.is_empty() || values.len() > cmd.positional_params.len() {
        return output;
    }
    output.extend(comp_positional(&cmd.positional_params[values.len() - 1]));
    output
}

fn comp_subcomands(cmd: &Command) -> Vec<(String, String)> {
    let mut output = vec![];
    for subcmd in cmd.subcommands.iter() {
        let describe = subcmd.describe.clone();
        for v in subcmd.list_names() {
            output.push((v, describe.clone()))
        }
    }
    if !output.is_empty() {
        output.push(("help".into(), String::new()));
    }
    output
}

fn comp_flag_option(param: &FlagOptionParam, index: usize) -> Vec<(String, String)> {
    let value_name = param
        .arg_value_names
        .get(index)
        .map(|v| v.as_str())
        .unwrap();
    comp_param(
        &param.describe,
        value_name,
        &param.choices,
        &param.choices_fn,
        param.multiple,
        param.required,
    )
}

fn comp_positional(param: &PositionalParam) -> Vec<(String, String)> {
    comp_param(
        &param.describe,
        &param.arg_value_name,
        &param.choices,
        &param.choices_fn,
        param.multiple,
        param.required,
    )
}

fn comp_param(
    describe: &str,
    value_name: &str,
    choices: &Option<Vec<String>>,
    choices_fn: &Option<String>,
    multiple: bool,
    required: bool,
) -> Vec<(String, String)> {
    let choices: Option<Either<Vec<String>, String>> = if let Some(choices_fn) = choices_fn {
        Some(Either::Right(choices_fn.to_string()))
    } else {
        choices
            .as_ref()
            .map(|choices| Either::Left(choices.iter().map(|v| v.to_string()).collect()))
    };
    if let Some(choices) = choices {
        match choices {
            Either::Left(choices) => choices
                .iter()
                .map(|v| (v.to_string(), String::new()))
                .collect(),
            Either::Right(choices_fn) => vec![(format!("__argc_fn:{}", choices_fn), String::new())],
        }
    } else {
        let value = match (multiple, required) {
            (true, true) => format!("__argc_value+{}", value_name),
            (true, false) => format!("__argc_value*{}", value_name),
            (false, true) => format!("__argc_value!{}", value_name),
            (false, false) => format!("__argc_value:{}", value_name),
        };
        vec![(value, describe.into())]
    }
}

fn get_param_choices<'a, 'b: 'a>(
    choices: &'a Option<Vec<String>>,
    choices_fn: &'a Option<String>,
    choices_values: &'a HashMap<&str, Vec<String>>,
) -> Option<&'a Vec<String>> {
    choices.as_ref().or_else(|| {
        choices_fn
            .as_ref()
            .and_then(|fn_name| choices_values.get(fn_name.as_str()))
    })
}
