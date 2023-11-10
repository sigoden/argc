#![allow(clippy::too_many_arguments)]

use std::{
    collections::{HashMap, HashSet},
    env,
    path::MAIN_SEPARATOR,
};

use crate::{
    command::{Command, SymbolParam},
    compgen::CompColor,
    param::{ChoiceData, FlagOptionParam, ParamData, PositionalParam},
    utils::run_param_fns,
    ArgcValue, Shell,
};

use either::Either;
use indexmap::{IndexMap, IndexSet};

const KNOWN_OPTIONS: [&str; 6] = ["-h", "-help", "--help", "-V", "-version", "--version"];

pub(crate) struct Matcher<'a, 'b> {
    cmds: Vec<LevelCommand<'a, 'b>>,
    args: &'b [String],
    flag_option_args: Vec<Vec<FlagOptionArg<'a, 'b>>>,
    positional_args: Vec<&'b str>,
    symbol_args: Vec<SymbolArg<'a, 'b>>,
    dashes: Option<usize>,
    arg_comp: ArgComp,
    choice_fns: HashSet<&'a str>,
    script_path: Option<String>,
    term_width: Option<usize>,
    split_last_arg_at: Option<usize>,
    last_flag_option: Option<&'a str>,
}

type FlagOptionArg<'a, 'b> = (&'b str, Vec<&'b str>, Option<&'a str>); // key, values, param_name
type SymbolArg<'a, 'b> = (&'b str, &'a SymbolParam);
type LevelCommand<'a, 'b> = (&'b str, &'a Command, usize); // name, command, arg_index

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArgComp {
    FlagOrOption,
    FlagOrOptionCombine(String),
    CommandOrPositional,
    OptionValue(String, usize),
    Symbol(char),
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
    pub(crate) fn new(root: &'a Command, args: &'b [String], compgen: bool) -> Self {
        let combine_shorts = root.has_metadata("combine-shorts");
        let mut cmds: Vec<LevelCommand> = vec![(args[0].as_str(), root, 0)];
        let mut cmd_level = 0;
        let mut arg_index = 1;
        let mut flag_option_args = vec![vec![]];
        let mut positional_args = vec![];
        let mut symbol_args = vec![];
        let mut dashes = None;
        let mut split_last_arg_at = None;
        let mut arg_comp = ArgComp::Any;
        let mut choice_fns = HashSet::new();
        let mut last_flag_option = None;
        let args_len = args.len();
        if root.delegated() {
            positional_args = args.iter().skip(1).map(|v| v.as_str()).collect();
        } else {
            let mut is_rest_args_positional = false; // option(e.g. -f --foo) will be treated as positional arg
            if let Some(arg) = args.last() {
                if arg.starts_with(|c| root.flag_option_signs().contains(c)) {
                    arg_comp = ArgComp::FlagOrOption;
                } else if !arg.is_empty() {
                    arg_comp = ArgComp::CommandOrPositional;
                }
            }
            while arg_index < args_len {
                let cmd = cmds[cmd_level].1;
                let signs = cmd.flag_option_signs();
                let arg = args[arg_index].as_str();
                let is_last_arg = arg_index == args_len - 1;
                last_flag_option = None;
                if arg == "--" {
                    if is_rest_args_positional {
                        add_positional_arg(
                            &mut positional_args,
                            arg,
                            &mut is_rest_args_positional,
                            cmd,
                        );
                    }
                    if dashes.is_none() {
                        dashes = Some(positional_args.len());
                        if !is_last_arg {
                            is_rest_args_positional = true
                        }
                    }
                } else if is_rest_args_positional
                    || (cmd.no_flags_options_subcommands() && !KNOWN_OPTIONS.contains(&arg))
                {
                    add_positional_arg(
                        &mut positional_args,
                        arg,
                        &mut is_rest_args_positional,
                        cmd,
                    );
                } else if arg.starts_with(|c| signs.contains(c)) {
                    if let Some((k, v)) = arg.split_once('=') {
                        if let Some(param) = cmd.find_flag_option(k) {
                            add_param_choice_fn(&mut choice_fns, param);
                            if is_last_arg {
                                arg_comp = ArgComp::OptionValue(param.var_name().to_string(), 0);
                                split_last_arg_at = Some(k.len() + 1);
                            }
                            flag_option_args[cmd_level].push((k, vec![v], Some(param.var_name())));
                            last_flag_option = Some(param.var_name());
                        } else if let Some((param, prefix)) = cmd.find_prefixed_option(arg) {
                            add_param_choice_fn(&mut choice_fns, param);
                            match_prefix_option(
                                &mut flag_option_args[cmd_level],
                                args,
                                &mut arg_index,
                                param,
                                &mut arg_comp,
                                &mut split_last_arg_at,
                                &prefix,
                            );
                            last_flag_option = Some(param.var_name());
                        } else {
                            flag_option_args[cmd_level].push((k, vec![v], None));
                        }
                    } else if let Some(param) = cmd.find_flag_option(arg) {
                        add_param_choice_fn(&mut choice_fns, param);
                        match_flag_option(
                            &mut flag_option_args[cmd_level],
                            args,
                            &mut arg_index,
                            param,
                            &mut arg_comp,
                            &mut split_last_arg_at,
                            combine_shorts,
                            &signs,
                        );
                        last_flag_option = Some(param.var_name());
                    } else if let Some((param, prefix)) = cmd.find_prefixed_option(arg) {
                        add_param_choice_fn(&mut choice_fns, param);
                        match_prefix_option(
                            &mut flag_option_args[cmd_level],
                            args,
                            &mut arg_index,
                            param,
                            &mut arg_comp,
                            &mut split_last_arg_at,
                            &prefix,
                        );
                        last_flag_option = Some(param.var_name());
                    } else if let Some(subcmd) = find_subcommand(cmd, arg, &positional_args)
                        .and_then(|v| {
                            if is_last_arg && compgen {
                                None
                            } else {
                                Some(v)
                            }
                        })
                    {
                        match_command(
                            &mut cmds,
                            &mut cmd_level,
                            &mut flag_option_args,
                            subcmd,
                            arg,
                            arg_index,
                            &mut is_rest_args_positional,
                        );
                    } else if let Some((mut arr, maybe_subcmd)) =
                        match_combine_shorts(cmd, arg, combine_shorts)
                    {
                        let mut current_cmd = cmd;
                        if let Some(subcmd) = maybe_subcmd {
                            match_command(
                                &mut cmds,
                                &mut cmd_level,
                                &mut flag_option_args,
                                subcmd,
                                arg,
                                arg_index,
                                &mut is_rest_args_positional,
                            );
                            current_cmd = subcmd;
                        }

                        if arr.is_empty() {
                            // match subcommand
                            if combine_shorts {
                                arg_comp = ArgComp::FlagOrOptionCombine(arg.to_string());
                            }
                        } else {
                            let name = arr.pop().and_then(|v| v.2).unwrap();
                            let param = current_cmd.find_flag_option(name).unwrap();
                            add_param_choice_fn(&mut choice_fns, param);
                            flag_option_args[cmd_level].extend(arr);
                            match_flag_option(
                                &mut flag_option_args[cmd_level],
                                args,
                                &mut arg_index,
                                param,
                                &mut arg_comp,
                                &mut split_last_arg_at,
                                combine_shorts,
                                &signs,
                            );
                            last_flag_option = Some(param.var_name());
                        }
                    } else if let Some((ch, symbol_param)) = find_symbol(cmd, arg) {
                        if let Some(choice_fn) = &symbol_param.1 {
                            choice_fns.insert(choice_fn);
                        }
                        symbol_args.push((&arg[1..], symbol_param));
                        if is_last_arg {
                            arg_comp = ArgComp::Symbol(ch);
                            split_last_arg_at = Some(1);
                        }
                    } else {
                        flag_option_args[cmd_level].push((arg, vec![], None));
                    }
                } else if let Some(subcmd) =
                    find_subcommand(cmd, arg, &positional_args).and_then(|v| {
                        if is_last_arg && compgen {
                            None
                        } else {
                            Some(v)
                        }
                    })
                {
                    match_command(
                        &mut cmds,
                        &mut cmd_level,
                        &mut flag_option_args,
                        subcmd,
                        arg,
                        arg_index,
                        &mut is_rest_args_positional,
                    );
                } else if let Some((ch, symbol_param)) = find_symbol(cmd, arg) {
                    if let Some(choice_fn) = &symbol_param.1 {
                        choice_fns.insert(choice_fn);
                    }
                    symbol_args.push((&arg[1..], symbol_param));
                    if is_last_arg {
                        arg_comp = ArgComp::Symbol(ch);
                        split_last_arg_at = Some(1);
                    }
                } else {
                    add_positional_arg(
                        &mut positional_args,
                        arg,
                        &mut is_rest_args_positional,
                        cmd,
                    );
                }
                arg_index += 1;
            }

            if is_rest_args_positional {
                arg_comp = ArgComp::CommandOrPositional;
            }

            let last_cmd = cmds.last().unwrap().1;
            choice_fns.extend(last_cmd.positional_params.iter().filter_map(|v| {
                if let Some((choice_fn, validate)) = v.choice_fn() {
                    if *validate {
                        return Some(choice_fn.as_str());
                    }
                }
                None
            }));
        }
        Self {
            cmds,
            args,
            flag_option_args,
            positional_args,
            symbol_args,
            dashes,
            arg_comp,
            choice_fns,
            script_path: None,
            term_width: None,
            split_last_arg_at,
            last_flag_option,
        }
    }

    pub(crate) fn set_script_path(&mut self, script_path: &str) {
        self.script_path = Some(script_path.to_string());
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
        if let Some(idx) = self.dashes {
            output.push(ArgcValue::Single("_dashes".into(), idx.to_string()));
        }
        let last_cmd = self.cmds[self.cmds.len() - 1].1;
        if let Some(name) = &last_cmd.name {
            output.push(ArgcValue::Single("_cmd_fn".into(), name.to_string()));
        }
        if let Some(name) = self.last_flag_option {
            output.push(ArgcValue::Single(
                "_last_flag_option".into(),
                name.to_string(),
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
            return vec![(
                "__argc_value=path".into(),
                String::new(),
                false,
                CompColor::of_value(),
            )];
        }
        let level = self.cmds.len() - 1;
        let last_cmd = self.cmds[level].1;
        if last_cmd.delegated() {
            return last_cmd
                .positional_params
                .first()
                .map(comp_positional)
                .unwrap_or_default();
        }
        let mut output = match &self.arg_comp {
            ArgComp::FlagOrOption => {
                let mut output = self.comp_flag_options();
                output.extend(comp_subcomands(last_cmd, true));
                output
            }
            ArgComp::FlagOrOptionCombine(value) => {
                let mut output: Vec<CompItem> = vec![];
                if value.len() == 2 && self.cmds[level].0 == value {
                    output.extend(comp_subcomands(self.cmds[level - 1].1, true));
                }
                output.extend(self.comp_flag_options().iter().filter_map(
                    |(v, description, nospace, comp_color)| {
                        if v.len() == 2 && v != value {
                            Some((
                                format!("{value}{}", &v[1..]),
                                description.to_string(),
                                *nospace,
                                *comp_color,
                            ))
                        } else {
                            None
                        }
                    },
                ));
                if output.len() == 1 {
                    output.insert(
                        0,
                        (
                            value.to_string(),
                            String::new(),
                            false,
                            CompColor::of_flag(),
                        ),
                    );
                }
                output
            }
            ArgComp::CommandOrPositional => {
                if self.positional_args.len() == 2 && self.positional_args[0] == "help" {
                    return comp_subcomands(last_cmd, false);
                }
                let values = self.match_positionals();
                comp_subcommands_positional(last_cmd, &values, self.positional_args.len() < 2)
            }
            ArgComp::OptionValue(name, index) => {
                if let Some(param) = last_cmd
                    .flag_option_params
                    .iter()
                    .find(|v| v.var_name() == name)
                {
                    comp_flag_option(param, *index)
                } else {
                    vec![]
                }
            }
            ArgComp::Symbol(ch) => comp_symbol(last_cmd, *ch),
            ArgComp::Any => {
                if self.positional_args.len() == 2 && self.positional_args[0] == "help" {
                    return comp_subcomands(last_cmd, false);
                }
                let values = self.match_positionals();
                comp_subcommands_positional(last_cmd, &values, self.positional_args.len() < 2)
            }
        };
        if output.is_empty()
            && !self.arg_comp.is_flag_or_option()
            && last_cmd.positional_params.is_empty()
        {
            output.push((
                "__argc_value=path".into(),
                String::new(),
                false,
                CompColor::of_value(),
            ));
        }
        output
    }

    pub(crate) fn split_last_arg_at(&self) -> Option<usize> {
        self.split_last_arg_at
    }

    fn to_arg_values_base(&self) -> Vec<ArgcValue> {
        let mut output = vec![];
        let cmds_len = self.cmds.len();
        let level = cmds_len - 1;
        let last_cmd = self.cmds[level].1;
        let cmd_arg_index = self.cmds[level].2;

        for (arg, (name, _)) in self.symbol_args.iter() {
            output.push(ArgcValue::Single(name.to_string(), arg.to_string()));
        }

        for level in 0..cmds_len {
            let args = &self.flag_option_args[level];
            let cmd = self.cmds[level].1;
            for param in cmd.flag_option_params.iter() {
                let values: Vec<&[&str]> = args
                    .iter()
                    .filter_map(|(_, value, name)| {
                        if let Some(true) = name.map(|v| param.var_name() == v) {
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
        output.push(ArgcValue::Single(
            "_cmd_arg_index".into(),
            cmd_arg_index.to_string(),
        ));
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
                .filter(|param| param.required())
                .map(|v| v.render_value())
                .collect()
        } else {
            vec![]
        };

        let choices_fn_values = self.run_choices_fns().unwrap_or_default();

        for (i, param) in last_cmd.positional_params.iter().enumerate() {
            if let (Some(values), Some(choices)) = (
                positional_values.get(i),
                get_param_choice(&param.data.choice, &choices_fn_values),
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
                .filter(|v| v.required())
                .map(|v| v.var_name())
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
                    .filter_map(|v| cmd.find_flag_option(v).map(|v| v.render_name_notations()))
                    .collect();
                missing_params.extend(missing_flag_options)
            }
            for (name, indexes) in flag_option_map {
                if let Some(param) = cmd.flag_option_params.iter().find(|v| v.var_name() == name) {
                    let values_list: Vec<&[&str]> =
                        indexes.iter().map(|v| args[*v].1.as_slice()).collect();
                    if !param.multi_occurs() && values_list.len() > 1 {
                        return Some(MatchError::NotMultipleArgument(level, param.render_name()));
                    }
                    for values in values_list.iter() {
                        if param.is_flag() && !values.is_empty() {
                            return Some(MatchError::NoMoreValue(
                                level,
                                param.render_name(),
                                values[0].to_string(),
                            ));
                        } else if !param.validate_args_len(values.len()) {
                            return Some(MatchError::MismatchValues(
                                level,
                                param.render_name_notations(),
                            ));
                        }
                        if let Some(choices) =
                            get_param_choice(&param.data.choice, &choices_fn_values)
                        {
                            for value in values.iter() {
                                if !choices.contains(&value.to_string()) {
                                    return Some(MatchError::InvalidValue(
                                        level,
                                        value.to_string(),
                                        param.render_first_notation(),
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

    fn run_choices_fns(&'a self) -> Option<HashMap<&'a str, Vec<String>>> {
        let script_path = self.script_path.as_ref()?;
        let mut choices_fn_values = HashMap::new();
        let fns: Vec<&str> = self.choice_fns.iter().copied().collect();
        let mut envs = HashMap::new();
        envs.insert("ARGC_OS".into(), env::consts::OS.to_string());
        envs.insert("ARGC_PATH_SEP".into(), MAIN_SEPARATOR.into());
        let outputs = run_param_fns(script_path, &fns, self.args, envs)?;
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
            choices_fn_values.insert(fns[i], choices);
        }
        Some(choices_fn_values)
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
            if param.multiple() {
                let dashes_at = self.dashes.unwrap_or_default();
                let takes = if param_index == 0
                    && dashes_at > 0
                    && params_len == 2
                    && cmd.positional_params[1].multiple()
                {
                    dashes_at
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
                    r###"error: invalid value for `{value}`

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
        let cmd_paths: Vec<&str> = self.cmds.iter().take(level + 1).map(|v| v.0).collect();
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
        let last = self.args.last().map(|v| v.as_str()).unwrap_or_default();
        for param in cmd.flag_option_params.iter() {
            let mut exist = args.contains(param.var_name());
            if !last.is_empty() && param.is_match(last) {
                exist = false;
            }
            if !exist || param.multi_occurs() {
                let describe = param.describe_oneline();
                let kind = if param.is_flag() {
                    CompColor::of_flag()
                } else {
                    CompColor::of_option()
                };
                for v in param.list_option_names() {
                    output.push((v, describe.to_string(), param.prefixed().is_some(), kind))
                }
            }
        }
        output
    }
}

/// (value, description, nospace, color)
pub(crate) type CompItem = (String, String, bool, CompColor);

fn find_subcommand<'a>(
    cmd: &'a Command,
    arg: &str,
    positional_args: &Vec<&str>,
) -> Option<&'a Command> {
    cmd.find_subcommand(arg).and_then(|v| {
        if positional_args.is_empty() {
            Some(v)
        } else {
            None
        }
    })
}

fn find_symbol<'a>(cmd: &'a Command, arg: &str) -> Option<(char, &'a SymbolParam)> {
    for (ch, param) in cmd.symbols.iter() {
        if arg.starts_with(*ch) {
            return Some((*ch, param));
        }
    }
    None
}

fn add_positional_arg<'a>(
    positional_args: &mut Vec<&'a str>,
    arg: &'a str,
    is_rest_args_positional: &mut bool,
    cmd: &Command,
) {
    positional_args.push(arg);
    if !*is_rest_args_positional
        && cmd.positional_params.last().map(|v| v.terminated()) == Some(true)
        && positional_args.len() >= cmd.positional_params.len() - 1
    {
        *is_rest_args_positional = true;
    }
}

fn take_value_args<'a>(args: &'a [String], start: usize, len: usize, signs: &str) -> Vec<&'a str> {
    let mut output = vec![];
    if len == 0 {
        return output;
    }
    let end = (start + len).min(args.len());
    for arg in args.iter().take(end).skip(start) {
        if arg.starts_with(|c| signs.contains(c)) {
            break;
        }
        output.push(arg.as_str());
    }
    output
}

fn match_combine_shorts<'a, 'b>(
    cmd: &'a Command,
    arg: &'b str,
    combine_shorts: bool,
) -> Option<(Vec<FlagOptionArg<'a, 'b>>, Option<&'a Command>)> {
    if !combine_shorts {
        return None;
    }
    if arg.starts_with("--") || arg == "-" {
        return None;
    }
    let mut current_cmd = cmd;
    let mut subcmd = None;
    let mut output = vec![];
    for (i, ch) in arg.chars().skip(1).enumerate() {
        let name: String = format!("-{ch}");
        if i == 0 {
            if let Some(v) = cmd.find_subcommand(&name) {
                current_cmd = v;
                subcmd = Some(v);
                continue;
            }
        }
        if let Some(param) = current_cmd.find_flag_option(&name) {
            output.push((arg, vec![], Some(param.var_name())))
        } else {
            return None;
        }
    }

    Some((output, subcmd))
}

fn match_flag_option<'a, 'b>(
    flag_option_args: &mut Vec<FlagOptionArg<'a, 'b>>,
    args: &'b [String],
    arg_index: &mut usize,
    param: &'a FlagOptionParam,
    arg_comp: &mut ArgComp,
    split_last_arg_at: &mut Option<usize>,
    combine_shorts: bool,
    signs: &str,
) {
    if param.terminated() {
        let value_args: Vec<&str> = args[*arg_index + 1..].iter().map(|v| v.as_str()).collect();
        let arg = &args[*arg_index];
        *arg_index += value_args.len();
        if !value_args.is_empty() {
            *arg_comp = ArgComp::OptionValue(
                param.var_name().to_string(),
                value_args.len().saturating_sub(1),
            );
        }
        flag_option_args.push((arg, value_args, Some(param.var_name())));
    } else {
        let mut values_len = param.arg_value_names.len();
        if param.unlimited_args() {
            values_len = usize::MAX / 2;
        }
        let args_len = args.len();
        let value_args = take_value_args(args, *arg_index + 1, values_len, signs);
        let arg = &args[*arg_index];
        *arg_index += value_args.len();
        if *arg_index == args_len - 1 {
            if *arg_comp != ArgComp::FlagOrOption {
                if param.is_option() && value_args.len() <= values_len {
                    *arg_comp = ArgComp::OptionValue(
                        param.var_name().to_string(),
                        value_args.len().saturating_sub(1),
                    );
                }
            } else if let Some(prefix) = param.prefixed() {
                if arg.starts_with(&prefix) {
                    *arg_comp = ArgComp::OptionValue(param.var_name().to_string(), 0);
                    *split_last_arg_at = Some(prefix.len());
                }
            } else if combine_shorts && param.is_flag() && !(arg.len() > 2 && param.is_match(arg)) {
                *arg_comp = ArgComp::FlagOrOptionCombine(arg.to_string());
            }
        }
        flag_option_args.push((arg, value_args, Some(param.var_name())));
    }
}

fn match_prefix_option<'a, 'b>(
    flag_option_args: &mut Vec<FlagOptionArg<'a, 'b>>,
    args: &'b [String],
    arg_index: &mut usize,
    param: &'a FlagOptionParam,
    arg_comp: &mut ArgComp,
    split_last_arg_at: &mut Option<usize>,
    prefix: &str,
) {
    let prefix_len = prefix.len();
    let args_len = args.len();
    let arg = &args[*arg_index];
    if *arg_index == args_len - 1 {
        *arg_comp = ArgComp::OptionValue(param.var_name().to_string(), 0);
        *split_last_arg_at = Some(prefix_len);
    }
    flag_option_args.push((arg, vec![&arg[prefix_len..]], Some(param.var_name())));
}

fn match_command<'a, 'b>(
    cmds: &mut Vec<LevelCommand<'a, 'b>>,
    cmd_level: &mut usize,
    flag_option_args: &mut Vec<Vec<FlagOptionArg<'a, 'b>>>,
    subcmd: &'a Command,
    arg: &'b str,
    arg_index: usize,
    is_rest_args_positional: &mut bool,
) {
    if subcmd.delegated() {
        *is_rest_args_positional = true;
    }
    *cmd_level += 1;
    cmds.push((arg, subcmd, arg_index));
    flag_option_args.push(vec![]);
}

fn add_param_choice_fn<'a>(choice_fns: &mut HashSet<&'a str>, param: &'a FlagOptionParam) {
    if let Some((choice_fn, validate)) = param.choice_fn() {
        if *validate {
            choice_fns.insert(choice_fn.as_str());
        }
    }
}

fn comp_subcommands_positional(
    cmd: &Command,
    values: &[Vec<&str>],
    with_subcmd: bool,
) -> Vec<CompItem> {
    let mut output = vec![];
    if with_subcmd {
        output.extend(comp_subcomands(cmd, false))
    }
    if values.is_empty() || values.len() > cmd.positional_params.len() {
        return output;
    }
    output.extend(comp_positional(&cmd.positional_params[values.len() - 1]));
    output
}

fn comp_subcomands(cmd: &Command, flag: bool) -> Vec<CompItem> {
    let mut output = vec![];
    let mut has_help_subcmd = false;
    let mut describe_help_subcmd = false;
    let signs = cmd.flag_option_signs();
    for subcmd in cmd.subcommands.iter() {
        let describe = subcmd.describe_oneline();
        for (i, v) in subcmd.list_names().into_iter().enumerate() {
            if i > 0 && v.len() < 2 {
                continue;
            }
            if (flag && v.starts_with(|c| signs.contains(c)))
                || (!flag && !v.starts_with(|c| signs.contains(c)))
            {
                if !flag {
                    has_help_subcmd = true;
                }
                if !describe.is_empty() {
                    describe_help_subcmd = true
                }
                output.push((v, describe.to_string(), false, CompColor::of_command()))
            }
        }
    }
    if has_help_subcmd {
        let describe = if describe_help_subcmd {
            "Show help for a command"
        } else {
            ""
        };
        output.push((
            "help".to_string(),
            describe.to_string(),
            false,
            CompColor::of_command(),
        ))
    }
    output
}

fn comp_symbol(cmd: &Command, ch: char) -> Vec<CompItem> {
    if let Some((name, choices_fn)) = cmd.symbols.get(&ch) {
        match choices_fn {
            Some(choices_fn) => {
                vec![(
                    format!("__argc_fn={}", choices_fn),
                    String::new(),
                    false,
                    CompColor::of_value(),
                )]
            }
            None => {
                vec![(
                    format!("__argc_value={}", name),
                    String::new(),
                    false,
                    CompColor::of_value(),
                )]
            }
        }
    } else {
        vec![]
    }
}

fn comp_flag_option(param: &FlagOptionParam, index: usize) -> Vec<CompItem> {
    let value_name = param
        .arg_value_names
        .get(index)
        .map(|v| v.as_str())
        .unwrap_or_else(|| param.arg_value_names.last().unwrap());
    comp_param(param.describe_oneline(), value_name, &param.data)
}

fn comp_positional(param: &PositionalParam) -> Vec<CompItem> {
    comp_param(param.describe_oneline(), &param.arg_value_name, &param.data)
}

fn comp_param(describe: &str, value_name: &str, data: &ParamData) -> Vec<CompItem> {
    let choices: Option<Either<Vec<String>, String>> =
        if let Some((choice_fn, _)) = data.choice_fn() {
            Some(Either::Right(choice_fn.to_string()))
        } else {
            data.choice_values()
                .map(|choices| Either::Left(choices.iter().map(|v| v.to_string()).collect()))
        };
    let mut output = if let Some(choices) = choices {
        match choices {
            Either::Left(choices) => choices
                .iter()
                .map(|v| (v.to_string(), String::new(), false, CompColor::of_value()))
                .collect(),
            Either::Right(choices_fn) => vec![(
                format!("__argc_fn={}", choices_fn),
                String::new(),
                false,
                CompColor::of_value(),
            )],
        }
    } else {
        let value = format!("__argc_value={}", value_name);
        vec![(value, describe.into(), false, CompColor::of_value())]
    };
    if let Some(ch) = data.multi_char() {
        output.insert(
            0,
            (
                format!("__argc_multi={}", ch),
                String::new(),
                false,
                CompColor::of_value(),
            ),
        );
    }
    output
}

fn get_param_choice<'a, 'b: 'a>(
    choice: &'a Option<ChoiceData>,
    choices_fn_values: &'a HashMap<&str, Vec<String>>,
) -> Option<&'a Vec<String>> {
    match choice {
        Some(ChoiceData::Values(v)) => Some(v),
        Some(ChoiceData::Fn(choice_fn, validate)) => {
            if *validate {
                choices_fn_values.get(choice_fn.as_str())
            } else {
                None
            }
        }
        None => None,
    }
}
