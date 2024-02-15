#![allow(clippy::too_many_arguments)]

use std::{
    collections::{HashMap, HashSet},
    env,
};

use crate::{
    argc_value::ArgcValue,
    command::{Command, SymbolParam},
    compgen::CompColor,
    param::{ChoiceValue, FlagOptionParam, Param, ParamData, PositionalParam},
    utils::{argc_var_name, run_param_fns, META_COMBINE_SHORTS, META_DOTENV},
    Shell,
};

use either::Either;
use indexmap::{IndexMap, IndexSet};

pub(crate) struct Matcher<'a, 'b> {
    cmds: Vec<&'a Command>,
    cmd_arg_indexes: Vec<usize>,
    args: &'b [String],
    flag_option_args: Vec<Vec<FlagOptionArg<'a, 'b>>>,
    positional_args: Vec<&'b str>,
    symbol_args: Vec<SymbolArg<'a, 'b>>,
    dash: Option<usize>,
    arg_comp: ArgComp,
    choice_fns: HashSet<&'a str>,
    script_path: Option<String>,
    envs: HashMap<&'a str, String>,
    term_width: Option<usize>,
    split_last_arg_at: Option<usize>,
    comp_option: Option<&'a str>,
}

pub(crate) type FlagOptionArg<'a, 'b> = (&'b str, Vec<&'b str>, Option<&'a str>); // key, values, param_name
pub(crate) type SymbolArg<'a, 'b> = (&'b str, &'a SymbolParam);

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
    MissingRequiredArguments(usize, Vec<String>),
    MissingRequiredEnvironments(Vec<String>),
    NotMultipleArgument(usize, String),
    InvalidValue(usize, String, String, Vec<String>),
    InvalidEnvironment(usize, String, String, Vec<String>),
    MismatchValues(usize, String),
    NoFlagValue(usize, String),
}

impl<'a, 'b> Matcher<'a, 'b> {
    pub(crate) fn new(root_cmd: &'a Command, args: &'b [String], compgen: bool) -> Self {
        let combine_shorts = root_cmd.has_metadata(META_COMBINE_SHORTS);
        let mut cmds: Vec<&'a Command> = vec![root_cmd];
        let mut cmd_level = 0;
        let mut cmd_arg_indexes = vec![0];
        let mut arg_index = 1;
        let mut flag_option_args = vec![vec![]];
        let mut positional_args = vec![];
        let mut symbol_args = vec![];
        let mut dash = None;
        let mut split_last_arg_at = None;
        let mut arg_comp = ArgComp::Any;
        let mut choice_fns = HashSet::new();
        let mut comp_option = None;
        let args_len = args.len();
        if root_cmd.delegated() {
            positional_args = args.iter().skip(1).map(|v| v.as_str()).collect();
        } else {
            let mut is_rest_args_positional = false; // option(e.g. -f --foo) will be treated as positional arg
            if let Some(arg) = args.last() {
                if arg.starts_with(|c| root_cmd.flag_option_signs().contains(c)) {
                    arg_comp = ArgComp::FlagOrOption;
                } else if !arg.is_empty() {
                    arg_comp = ArgComp::CommandOrPositional;
                }
            }
            while arg_index < args_len {
                let cmd = cmds[cmd_level];
                let signs = cmd.flag_option_signs();
                let arg = args[arg_index].as_str();
                let is_last_arg = arg_index == args_len - 1;
                comp_option = None;
                if arg == "--" {
                    if is_rest_args_positional {
                        add_positional_arg(
                            &mut positional_args,
                            arg,
                            &mut is_rest_args_positional,
                            cmd,
                        );
                    }
                    if dash.is_none() {
                        dash = Some(positional_args.len());
                        if !is_last_arg {
                            is_rest_args_positional = true
                        }
                    }
                } else if is_rest_args_positional
                    || (cmd.no_flags_options_subcommands()
                        && !cmd.help_flags().contains(&arg)
                        && !cmd.version_flags().contains(&arg))
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
                            let split_at = if let Some(prefix) = param.match_prefix(arg) {
                                prefix.len()
                            } else {
                                k.len() + 1
                            };
                            if is_last_arg {
                                arg_comp = ArgComp::OptionValue(param.id().to_string(), 0);
                                split_last_arg_at = Some(split_at);
                            }
                            let values = delimit_arg_values(param, &[v]);
                            flag_option_args[cmd_level].push((k, values, Some(param.id())));
                            comp_option = Some(param.id());
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
                        comp_option = Some(param.id());
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
                            &mut cmd_arg_indexes,
                            &mut flag_option_args,
                            subcmd,
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
                                &mut cmd_arg_indexes,
                                &mut flag_option_args,
                                subcmd,
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
                            comp_option = Some(param.id());
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
                        &mut cmd_arg_indexes,
                        &mut flag_option_args,
                        subcmd,
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
                    let mut target_cmd = cmd;
                    if positional_args.is_empty() && !(compgen && is_last_arg) {
                        if let Some(subcmd) = cmd.find_default_subcommand() {
                            match_command(
                                &mut cmds,
                                &mut cmd_level,
                                &mut cmd_arg_indexes,
                                &mut flag_option_args,
                                subcmd,
                                arg_index,
                                &mut is_rest_args_positional,
                            );
                            target_cmd = cmds[cmd_level];
                        }
                    }
                    add_positional_arg(
                        &mut positional_args,
                        arg,
                        &mut is_rest_args_positional,
                        target_cmd,
                    );
                }
                arg_index += 1;
            }

            if is_rest_args_positional {
                arg_comp = ArgComp::CommandOrPositional;
            }

            let last_cmd = *cmds.last().unwrap();
            for param in &last_cmd.positional_params {
                add_param_choice_fn(&mut choice_fns, param)
            }
        }

        let mut envs = HashMap::new();
        let last_cmd = *cmds.last().unwrap();
        for param in &last_cmd.env_params {
            if let Ok(value) = std::env::var(param.id()) {
                envs.insert(param.id(), value);
            }
            add_param_choice_fn(&mut choice_fns, param)
        }

        Self {
            cmds,
            cmd_arg_indexes,
            args,
            flag_option_args,
            positional_args,
            symbol_args,
            dash,
            arg_comp,
            choice_fns,
            script_path: None,
            term_width: None,
            split_last_arg_at,
            comp_option,
            envs,
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
        let last_cmd = self.last_cmd();
        let mut output = self.to_arg_values_base();
        if last_cmd.positional_params.is_empty() && !self.positional_args.is_empty() {
            output.push(ArgcValue::ExtraPositionalMultiple(
                self.positional_args.iter().map(|v| v.to_string()).collect(),
            ));
        }
        if let Some(command_fn) = &last_cmd.command_fn {
            output.push(ArgcValue::CommandFn(command_fn.clone()));
        }
        output
    }

    pub(crate) fn to_arg_values_for_param_fn(&self) -> Vec<ArgcValue> {
        let mut output: Vec<ArgcValue> = self.to_arg_values_base();
        let cmds_len = self.cmds.len();
        let level = cmds_len - 1;
        let last_cmd = self.cmds[level];
        let cmd_arg_index = self.cmd_arg_indexes[level];

        output.push(ArgcValue::Single(
            "_cmd_arg_index".into(),
            cmd_arg_index.to_string(),
        ));
        if let Some(name) = &last_cmd.match_fn {
            output.push(ArgcValue::Single("_cmd_fn".into(), name.to_string()));
        }
        if let Some(idx) = self.dash {
            output.push(ArgcValue::Single("_dash".into(), idx.to_string()));
        }
        if let Some(name) = self.comp_option {
            output.push(ArgcValue::Single("_option".into(), argc_var_name(name)));
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
        let last_cmd = self.cmds[level];
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
                if value.len() == 2 && &self.args[self.cmd_arg_indexes[level]] == value {
                    output.extend(comp_subcomands(self.cmds[level - 1], true));
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
                if let Some(param) = last_cmd.flag_option_params.iter().find(|v| v.id() == name) {
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
        let root_cmd = self.cmds[0];
        let cmds_len = self.cmds.len();
        let level = cmds_len - 1;
        let last_cmd = self.cmds[level];

        if let Some(value) = root_cmd.get_metadata(META_DOTENV) {
            output.push(ArgcValue::Dotenv(value.to_string()))
        }

        for param in &last_cmd.env_params {
            if !self.envs.contains_key(&param.id()) {
                if let Some(value) = param.get_env_value() {
                    output.push(value);
                }
            }
        }

        let (before_hook, after_hook) = last_cmd.exist_hooks();
        if before_hook || after_hook {
            output.push(ArgcValue::Hook((before_hook, after_hook)));
        }

        for (arg, (name, _)) in self.symbol_args.iter() {
            output.push(ArgcValue::Single(name.to_string(), arg.to_string()));
        }

        for level in 0..cmds_len {
            let args = &self.flag_option_args[level];
            let cmd = self.cmds[level];
            for param in cmd.flag_option_params.iter() {
                if let Some(value) = param.to_argc_value(args) {
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
            if let Some(value) = param.to_argc_value(values) {
                output.push(value);
            }
        }
        output.push(ArgcValue::Multiple("_args".into(), self.args.to_vec()));
        output
    }

    fn validate(&self) -> Option<MatchError> {
        let cmds_len = self.cmds.len();
        let choices_fn_values = self.run_choices_fns().unwrap_or_default();

        for level in 0..cmds_len {
            let flag_option_args = &self.flag_option_args[level];
            let cmd = self.cmds[level];
            let mut flag_option_map = IndexMap::new();
            let mut missing_flag_options: IndexSet<&str> = cmd
                .flag_option_params
                .iter()
                .filter(|v| v.required())
                .map(|v| v.id())
                .collect();
            for (i, (key, _, name)) in flag_option_args.iter().enumerate() {
                match (*key, name) {
                    ("--help", _) | ("-help", _) | ("-h", None) | (_, Some("help")) => {
                        return Some(MatchError::DisplayHelp)
                    }
                    ("--version", _) | ("-version", _) | ("-V", None) | (_, Some("version"))
                        if cmd.exist_version() =>
                    {
                        return Some(MatchError::DisplayVersion)
                    }
                    _ => {}
                }
                match *name {
                    Some(name) => {
                        missing_flag_options.swap_remove(name);
                        flag_option_map.entry(name).or_insert(vec![]).push(i);
                    }
                    None => return Some(MatchError::UnknownArgument(level, key.to_string())),
                }
            }
            for (name, indexes) in flag_option_map {
                if let Some(param) = cmd.flag_option_params.iter().find(|v| v.id() == name) {
                    let values_list: Vec<&[&str]> = indexes
                        .iter()
                        .map(|v| flag_option_args[*v].1.as_slice())
                        .collect();
                    let (min, _) = param.args_range();
                    for values in values_list.iter() {
                        if param.is_flag() && !values.is_empty() {
                            return Some(MatchError::NoFlagValue(level, param.render_long_name()));
                        } else if values.len() < min {
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
                    if !param.multiple_occurs() && values_list.len() > 1 {
                        return Some(MatchError::NotMultipleArgument(
                            level,
                            param.render_long_name(),
                        ));
                    }
                }
            }
            if !missing_flag_options.is_empty() {
                let missing_flag_options: Vec<String> = missing_flag_options
                    .iter()
                    .filter_map(|v| cmd.find_flag_option(v).map(|v| v.render_name_notations()))
                    .collect();
                return Some(MatchError::MissingRequiredArguments(
                    level,
                    missing_flag_options,
                ));
            }
        }

        let level = cmds_len - 1;
        let last_cmd = self.cmds[level];
        let last_args = &self.flag_option_args[level];
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
            if last_cmd.command_fn.is_none() {
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
        if positional_params_len > positional_values_len {
            let missing_positionals: Vec<_> = last_cmd.positional_params[positional_values_len..]
                .iter()
                .filter(|param| param.required())
                .map(|v| v.render_value())
                .collect();
            if !missing_positionals.is_empty() {
                return Some(MatchError::MissingRequiredArguments(
                    level,
                    missing_positionals,
                ));
            }
        }

        let mut missing_envs = vec![];
        for param in &last_cmd.env_params {
            if param.required() && !self.envs.contains_key(param.id()) {
                missing_envs.push(param.id().to_string());
            }
        }
        if !missing_envs.is_empty() {
            return Some(MatchError::MissingRequiredEnvironments(missing_envs));
        }

        for param in &last_cmd.env_params {
            if let (Some(choices), Some(value)) = (
                get_param_choice(&param.data.choice, &choices_fn_values),
                self.envs.get(param.id()),
            ) {
                if !choices.contains(&value.to_string()) {
                    return Some(MatchError::InvalidEnvironment(
                        level,
                        value.to_string(),
                        param.id().to_string(),
                        choices.clone(),
                    ));
                }
            }
        }

        None
    }

    fn run_choices_fns(&'a self) -> Option<HashMap<&'a str, Vec<String>>> {
        let script_path = self.script_path.as_ref()?;
        let mut choices_fn_values = HashMap::new();
        let fns: Vec<&str> = self.choice_fns.iter().copied().collect();
        let mut envs = HashMap::new();
        envs.insert("ARGC_OS".into(), env::consts::OS.to_string());
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
        let last_cmd = self.last_cmd();
        let params_len = last_cmd.positional_params.len();
        let mut arg_index = 0;
        let mut param_index = 0;
        while param_index < params_len && arg_index < args_len {
            let param = &last_cmd.positional_params[param_index];
            let takes = if param.multiple_values() {
                let dash = self.dash.unwrap_or_default();
                if param_index == 0
                    && dash > 0
                    && params_len == 2
                    && last_cmd.positional_params[1].multiple_values()
                {
                    dash
                } else {
                    (args_len - arg_index).saturating_sub(params_len - param_index) + 1
                }
            } else {
                1
            };
            let values =
                delimit_arg_values(param, &self.positional_args[arg_index..(arg_index + takes)]);
            output.push(values);
            arg_index += takes;
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
                let cmd = self.last_cmd();
                cmd.render_help(self.term_width)
            }
            MatchError::DisplaySubcommandHelp(name) => {
                let cmd = self.last_cmd();
                let cmd = cmd.find_subcommand(name).unwrap();
                cmd.render_help(self.term_width)
            }
            MatchError::DisplayVersion => {
                let cmd = self.last_cmd();
                cmd.render_version()
            }
            MatchError::InvalidSubcommand => {
                exit = 1;
                let cmd = self.last_cmd();
                let cmd_str = cmd.cmd_paths().join("-");
                let names = cmd.list_subcommand_names().join(", ");
                format!(
                    r###"error: `{cmd_str}` requires a subcommand but one was not provided
  [subcommands: {names}]"###
                )
            }
            MatchError::UnknownArgument(_level, name) => {
                exit = 1;
                format!(r###"error: unexpected argument `{name}` found"###)
            }
            MatchError::MissingRequiredArguments(_level, values) => {
                exit = 1;
                let list = values
                    .iter()
                    .map(|v| format!("  {v}"))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    r###"error: the following required arguments were not provided:
{list}"###
                )
            }
            MatchError::MissingRequiredEnvironments(values) => {
                exit = 1;
                let list = values
                    .iter()
                    .map(|v| format!("  {v}"))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    r###"error: the following required environments were not provided:
{list}"###
                )
            }
            MatchError::NotMultipleArgument(_level, name) => {
                exit = 1;
                format!(r###"error: the argument `{name}` cannot be used multiple times"###)
            }
            MatchError::InvalidValue(_level, value, name, choices) => {
                exit = 1;
                let list = choices.join(", ");
                format!(
                    r###"error: invalid value `{value}` for `{name}`
  [possible values: {list}]"###
                )
            }
            MatchError::InvalidEnvironment(_level, value, name, choices) => {
                exit = 1;
                let list = choices.join(", ");
                format!(
                    r###"error: invalid value `{value}` for environment variable `{name}`
  [possible values: {list}]"###
                )
            }
            MatchError::MismatchValues(_level, value) => {
                exit = 1;
                format!(r###"error: incorrect number of values for `{value}`"###)
            }
            MatchError::NoFlagValue(_level, name) => {
                exit = 1;
                format!(r###"error: flag `{name}` don't accept any value"###)
            }
        };
        (message, exit)
    }

    fn comp_flag_options(&self) -> Vec<CompItem> {
        let mut output = vec![];
        let level = self.cmds.len() - 1;
        let last_cmd = self.last_cmd();
        let args: HashSet<&str> = self.flag_option_args[level]
            .iter()
            .filter_map(|v| v.2)
            .collect();
        let last = self.args.last().map(|v| v.as_str()).unwrap_or_default();
        for param in last_cmd.flag_option_params.iter() {
            let mut exist = args.contains(param.id());
            if !last.is_empty() && param.is_match(last) {
                exist = false;
            }
            if !exist || param.multiple_occurs() {
                let describe = param.describe_oneline();
                let kind = if param.is_flag() {
                    CompColor::of_flag()
                } else {
                    CompColor::of_option()
                };
                for v in param.list_names() {
                    output.push((v, describe.to_string(), param.prefixed(), kind))
                }
            }
        }
        output
    }

    fn last_cmd(&self) -> &Command {
        self.cmds.last().unwrap()
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
            output.push((arg, vec![], Some(param.id())))
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
    let arg = &args[*arg_index];
    if param.terminated() {
        let value_args: Vec<&str> = args[*arg_index + 1..].iter().map(|v| v.as_str()).collect();
        *arg_index += value_args.len();
        if !value_args.is_empty() {
            *arg_comp =
                ArgComp::OptionValue(param.id().to_string(), value_args.len().saturating_sub(1));
        }
        flag_option_args.push((arg, value_args, Some(param.id())));
    } else if let Some(prefix) = param.match_prefix(arg) {
        let args_len = args.len();
        let prefix_len = prefix.len();
        let value_args = take_value_args(args, *arg_index + 1, 1, signs);
        let take_empty = value_args.is_empty();
        *arg_index += value_args.len();
        let values = if take_empty { vec!["1"] } else { value_args };
        if *arg_index == args_len - 1 {
            *arg_comp = ArgComp::OptionValue(param.id().to_string(), 0);
            if take_empty {
                *split_last_arg_at = Some(prefix_len);
            }
        }
        flag_option_args.push((arg, values, Some(param.id())));
    } else {
        let values_max = param.args_range().1;
        let args_len = args.len();
        let value_args = take_value_args(args, *arg_index + 1, values_max, signs);
        *arg_index += value_args.len();
        if *arg_index == args_len - 1 {
            if *arg_comp != ArgComp::FlagOrOption {
                if param.is_option() && value_args.len() <= values_max {
                    *arg_comp = ArgComp::OptionValue(
                        param.id().to_string(),
                        value_args.len().saturating_sub(1),
                    );
                }
            } else if combine_shorts && param.is_flag() && !(arg.len() > 2 && param.is_match(arg)) {
                *arg_comp = ArgComp::FlagOrOptionCombine(arg.to_string());
            }
        }
        let values = delimit_arg_values(param, &value_args);
        flag_option_args.push((arg, values, Some(param.id())));
    }
}

fn match_command<'a>(
    cmds: &mut Vec<&'a Command>,
    cmd_level: &mut usize,
    cmd_arg_indexes: &mut Vec<usize>,
    flag_option_args: &mut Vec<Vec<FlagOptionArg<'a, '_>>>,
    subcmd: &'a Command,
    arg_index: usize,
    is_rest_args_positional: &mut bool,
) {
    if subcmd.delegated() {
        *is_rest_args_positional = true;
    }
    *cmd_level += 1;
    cmds.push(subcmd);
    cmd_arg_indexes.push(arg_index);
    flag_option_args.push(vec![]);
}

fn add_param_choice_fn<'a>(choice_fns: &mut HashSet<&'a str>, param: &'a impl Param) {
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
        output.extend(comp_subcomands(cmd, false));

        // default subcommand
        if let Some(subcmd) = cmd.find_default_subcommand() {
            if !subcmd.positional_params.is_empty() {
                output.extend(comp_positional(&subcmd.positional_params[0]));
            }
        }
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
        .notations
        .get(index)
        .map(|v| v.as_str())
        .unwrap_or_else(|| param.notations.last().unwrap());
    comp_param(param.describe_oneline(), value_name, &param.data)
}

fn comp_positional(param: &PositionalParam) -> Vec<CompItem> {
    comp_param(param.describe_oneline(), &param.notation, &param.data)
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
    if let Some(ch) = data.args_delimiter() {
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
    choice: &'a Option<ChoiceValue>,
    choices_fn_values: &'a HashMap<&str, Vec<String>>,
) -> Option<&'a Vec<String>> {
    match choice {
        Some(ChoiceValue::Values(v)) => Some(v),
        Some(ChoiceValue::Fn(choice_fn, validate)) => {
            if *validate {
                choices_fn_values.get(choice_fn.as_str())
            } else {
                None
            }
        }
        None => None,
    }
}

fn delimit_arg_values<'b, T: Param>(param: &T, values: &[&'b str]) -> Vec<&'b str> {
    if let Some(delimiter) = param.args_delimiter() {
        values.iter().flat_map(|v| v.split(delimiter)).collect()
    } else {
        values.to_vec()
    }
}
