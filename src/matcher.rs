#![allow(clippy::too_many_arguments)]

use std::collections::{HashMap, HashSet};

use crate::{
    argc_value::ArgcValue,
    command::{Command, SymbolParam},
    param::{ChoiceValue, FlagOptionParam, Param, ParamData, PositionalParam},
    runtime::Runtime,
    utils::{argc_var_name, is_true_value, META_COMBINE_SHORTS},
};

#[cfg(feature = "compgen")]
use crate::{
    compgen::{CompColor, CompItem},
    Shell,
};

use either::Either;
use indexmap::{IndexMap, IndexSet};

pub(crate) struct Matcher<'a, 'b, T> {
    runtime: T,
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
    envs: HashMap<String, String>,
    wrap_width: Option<usize>,
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
    InvalidSubcommand(Option<String>),
    UnknownArgument(usize, String),
    MissingRequiredArguments(usize, Vec<String>),
    MissingRequiredEnvironments(Vec<String>),
    NotMultipleArgument(usize, String),
    InvalidValue(usize, String, String, Vec<String>),
    InvalidBindEnvironment(usize, String, String, String, Vec<String>),
    InvalidEnvironment(usize, String, String, Vec<String>),
    MismatchValues(usize, String),
    NoFlagValue(usize, String),
}

impl<'a, 'b, T: Runtime> Matcher<'a, 'b, T> {
    pub(crate) fn new(
        runtime: T,
        root_cmd: &'a Command,
        args: &'b [String],
        compgen: bool,
    ) -> Self {
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
                if maybe_flag_option(arg, &root_cmd.flag_option_signs()) {
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
                    || (cmd.is_empty_flags_options_subcommands()
                        && !cmd.help_flags.contains(&arg)
                        && !cmd.version_flags.contains(&arg))
                {
                    add_positional_arg(
                        &mut positional_args,
                        arg,
                        &mut is_rest_args_positional,
                        cmd,
                    );
                } else if arg.len() > 1 && maybe_flag_option(arg, &signs) {
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
                            if positional_args.is_empty()
                                && !cmd.help_flags.contains(&k)
                                && !cmd.version_flags.contains(&k)
                            {
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
                                    continue;
                                }
                            }
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
                            compgen,
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
                                compgen,
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
                        if positional_args.is_empty()
                            && !cmd.help_flags.contains(&arg)
                            && !cmd.version_flags.contains(&arg)
                        {
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
                                continue;
                            }
                        }
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

            let last_cmd = cmds[cmd_level];
            if positional_args.is_empty() && flag_option_args[cmd_level].is_empty() {
                if let Some(subcmd) = last_cmd.find_default_subcommand() {
                    if subcmd.command_fn.is_some() {
                        match_command(
                            &mut cmds,
                            &mut cmd_level,
                            &mut cmd_arg_indexes,
                            &mut flag_option_args,
                            subcmd,
                            arg_index - 1,
                            &mut is_rest_args_positional,
                        );
                    }
                }
            }

            for param in &last_cmd.positional_params {
                add_param_choice_fn(&mut choice_fns, param)
            }
        }

        let last_cmd = *cmds.last().unwrap();

        let mut envs = runtime.env_vars();
        if let Some(dot_envs) = root_cmd.dotenv().and_then(|v| runtime.load_dotenv(v)) {
            envs.extend(dot_envs)
        }

        for param in &last_cmd.env_params {
            add_param_choice_fn(&mut choice_fns, param)
        }

        Self {
            runtime,
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
            wrap_width: None,
            split_last_arg_at,
            comp_option,
            envs,
        }
    }

    pub(crate) fn set_script_path(&mut self, script_path: &str) {
        self.script_path = Some(script_path.to_string());
    }

    pub(crate) fn set_wrap_width(&mut self, wrap_width: usize) {
        self.wrap_width = Some(wrap_width);
    }

    #[cfg(feature = "eval")]
    pub(crate) fn to_arg_values(&self) -> Vec<ArgcValue> {
        let bind_envs = self.build_bind_envs();
        if let Some(err) = self.validate(&bind_envs) {
            return vec![ArgcValue::Error(self.stringify_match_error(&err))];
        }
        let last_cmd = self.last_cmd();
        let mut output = self.to_arg_values_base(&bind_envs);
        if last_cmd.positional_params.is_empty() && !self.positional_args.is_empty() {
            output.push(ArgcValue::ExtraPositionalMultiple(
                self.positional_args.iter().map(|v| v.to_string()).collect(),
            ));
        }
        if !last_cmd.require_tools.is_empty() {
            output.push(ArgcValue::RequireTools(
                last_cmd.require_tools.iter().cloned().collect(),
            ));
        }
        if let Some(command_fn) = &last_cmd.command_fn {
            output.push(ArgcValue::CommandFn(command_fn.clone()));
        }
        output
    }

    #[cfg(feature = "eval")]
    pub(crate) fn to_arg_values_for_param_fn(&self) -> Vec<ArgcValue> {
        let bind_envs = self.build_bind_envs();
        let mut output: Vec<ArgcValue> = self.to_arg_values_base(&bind_envs);
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

    #[cfg(feature = "compgen")]
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
                let mut output = self.comp_flag_options(false);
                output.extend(comp_subcomands(last_cmd, true));
                output
            }
            ArgComp::FlagOrOptionCombine(value) => {
                let mut output: Vec<CompItem> = vec![];
                if value.len() == 2 && &self.args[self.cmd_arg_indexes[level]] == value {
                    output.extend(comp_subcomands(self.cmds[level - 1], true));
                }
                output.extend(self.comp_flag_options(true).iter().filter_map(
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

    #[cfg(feature = "eval")]
    fn to_arg_values_base<'x>(&'x self, bind_envs: &BindEnvs<'a, 'x>) -> Vec<ArgcValue> {
        let mut output = vec![];
        let root_cmd = self.cmds[0];
        let cmds_len = self.cmds.len();
        let last_cmd = self.last_cmd();

        if let Some(value) = root_cmd.dotenv() {
            output.push(ArgcValue::Dotenv(value.to_string()))
        }

        for param in &last_cmd.env_params {
            if !self.envs.contains_key(param.id()) {
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
            let args = self.flag_option_args[level].as_slice();
            let cmd = self.cmds[level];
            for param in cmd.flag_option_params.iter() {
                let mut args: Vec<_> = args
                    .iter()
                    .filter_map(|(name, value, param_name)| {
                        if param_name == &Some(param.id()) {
                            Some((*name, value.as_slice()))
                        } else {
                            None
                        }
                    })
                    .collect();
                if args.is_empty() {
                    if let Some(env_values) = bind_envs.flag_options[level].get(param.id()) {
                        if param.is_flag() {
                            if is_true_value(env_values[0]) {
                                args = vec![("", env_values.as_slice())];
                            }
                        } else {
                            args = vec![("", env_values.as_slice())];
                        }
                    }
                }
                if let Some(value) = param.to_argc_value(&args) {
                    output.push(value);
                }
            }
        }

        let positional_values = self.match_positionals();
        for (i, param) in last_cmd.positional_params.iter().enumerate() {
            let mut values = positional_values
                .get(i)
                .map(|v| v.as_slice())
                .unwrap_or_default();
            if values.is_empty() {
                if let Some(env_values) = bind_envs.positionals.get(param.id()) {
                    values = env_values.as_slice()
                }
            }
            if let Some(value) = param.to_argc_value(values) {
                output.push(value);
            }
        }
        output.push(ArgcValue::Multiple("_args".into(), self.args.to_vec()));
        output
    }

    #[cfg(feature = "eval")]
    fn build_bind_envs<'x: 'a>(&'x self) -> BindEnvs<'a, 'x> {
        let cmds_len = self.cmds.len();
        let last_cmd = self.last_cmd();
        let mut bind_envs = BindEnvs::new(cmds_len);

        for level in 0..cmds_len {
            let cmd = self.cmds[level];
            for param in cmd.flag_option_params.iter() {
                if let Some(env_value) = param.bind_env().and_then(|v| self.envs.get(&v)) {
                    let values = delimit_arg_values(param, &[env_value]);
                    bind_envs.flag_options[level].insert(param.id(), values);
                    add_param_choice_fn(&mut bind_envs.choice_fns, param);
                }
            }
        }

        for param in last_cmd.positional_params.iter() {
            if let Some(env_value) = param.bind_env().and_then(|v| self.envs.get(&v)) {
                let values = delimit_arg_values(param, &[env_value]);
                bind_envs.positionals.insert(param.id(), values);
                add_param_choice_fn(&mut bind_envs.choice_fns, param);
            }
        }
        bind_envs
    }

    #[cfg(feature = "eval")]
    fn validate<'x>(&'x self, bind_envs: &BindEnvs<'a, 'x>) -> Option<MatchError> {
        let cmds_len = self.cmds.len();
        let choices_fn_values = self.execute_choices_fns(bind_envs).unwrap_or_default();
        for level in 0..cmds_len {
            let flag_option_args = &self.flag_option_args[level];
            let cmd = self.cmds[level];
            let flag_option_bind_envs = &bind_envs.flag_options[level];
            let mut flag_option_map = IndexMap::new();
            let mut missing_flag_options: IndexSet<&str> = cmd
                .flag_option_params
                .iter()
                .filter(|v| v.required() && !flag_option_bind_envs.contains_key(v.id()))
                .map(|v| v.id())
                .collect();

            let mut check_flag_option_bind_envs: IndexSet<&str> =
                flag_option_bind_envs.keys().copied().collect();
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
                        check_flag_option_bind_envs.swap_remove(name);
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
                    let (min, _) = param.num_args();
                    for values in values_list.iter() {
                        if param.is_flag() && !values.is_empty() {
                            return Some(MatchError::NoFlagValue(level, param.long_name()));
                        } else if values.len() < min {
                            return Some(MatchError::MismatchValues(
                                level,
                                param.render_name_notations(),
                            ));
                        }
                        if let Some(choices) = get_param_choice(param.choice(), &choices_fn_values)
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
                        return Some(MatchError::NotMultipleArgument(level, param.long_name()));
                    }
                }
            }

            for name in check_flag_option_bind_envs {
                if let Some(param) = cmd.flag_option_params.iter().find(|v| v.id() == name) {
                    let values = &flag_option_bind_envs[name];
                    let mut is_valid = true;
                    let mut choice_values = vec![];
                    let (min, _) = param.num_args();
                    if param.is_flag() {
                        if !is_bool_value(values[0]) {
                            is_valid = false;
                        }
                    } else if min > 1 {
                        is_valid = false;
                    }
                    if !is_valid {
                        return Some(MatchError::InvalidBindEnvironment(
                            level,
                            values[0].to_string(),
                            param.bind_env().unwrap_or_default(),
                            param.long_name(),
                            choice_values,
                        ));
                    }
                    if let Some(choices) = get_param_choice(param.choice(), &choices_fn_values) {
                        choice_values = choices.to_vec();
                        for value in values.iter() {
                            if !choices.contains(&value.to_string()) {
                                return Some(MatchError::InvalidBindEnvironment(
                                    level,
                                    value.to_string(),
                                    param.bind_env().unwrap_or_default(),
                                    param.long_name(),
                                    choice_values,
                                ));
                            }
                        }
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
                    return Some(MatchError::InvalidSubcommand(
                        self.positional_args.first().map(|v| v.to_string()),
                    ));
                }
            } else if last_cmd.positional_params.is_empty() && !self.positional_args.is_empty() {
                return Some(MatchError::InvalidSubcommand(
                    self.positional_args.first().map(|v| v.to_string()),
                ));
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
                get_param_choice(param.choice(), &choices_fn_values),
            ) {
                for value in values.iter() {
                    if !choices.contains(&value.to_string()) {
                        return Some(MatchError::InvalidValue(
                            level,
                            value.to_string(),
                            param.render_notation(),
                            choices.clone(),
                        ));
                    }
                }
            }
        }
        if positional_params_len > positional_values_len {
            let mut missing_positionals = vec![];
            for param in &last_cmd.positional_params[positional_values_len..] {
                if let Some(values) = bind_envs.positionals.get(param.id()) {
                    if let Some(choices) = get_param_choice(param.choice(), &choices_fn_values) {
                        for value in values.iter() {
                            if !choices.contains(&value.to_string()) {
                                return Some(MatchError::InvalidBindEnvironment(
                                    level,
                                    value.to_string(),
                                    param.bind_env().unwrap_or_default(),
                                    param.render_notation(),
                                    choices.to_vec(),
                                ));
                            }
                        }
                    }
                } else if param.required() {
                    missing_positionals.push(param.render_notation())
                }
            }
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
                get_param_choice(param.choice(), &choices_fn_values),
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

    #[cfg(feature = "eval")]
    fn execute_choices_fns<'x>(
        &'x self,
        bind_envs: &BindEnvs<'a, 'x>,
    ) -> Option<HashMap<&'a str, Vec<String>>> {
        let fns: Vec<_> = {
            let mut fns = self.choice_fns.clone();
            fns.extend(bind_envs.choice_fns.iter());
            fns.into_iter().collect()
        };
        let script_path = self.script_path.as_ref()?;
        let mut choices_fn_values = HashMap::new();
        let mut envs = HashMap::new();
        envs.insert("ARGC_OS".into(), self.runtime.os());
        let outputs = self
            .runtime
            .exec_bash_functions(script_path, &fns, self.args, envs)?;
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

    #[cfg(feature = "eval")]
    fn stringify_match_error(&self, err: &MatchError) -> (String, i32) {
        let mut exit = 0;
        let message = match err {
            MatchError::DisplayHelp => {
                let cmd = self.last_cmd();
                cmd.render_help(self.wrap_width)
            }
            MatchError::DisplaySubcommandHelp(name) => {
                let cmd = self.last_cmd();
                let cmd = cmd.find_subcommand(name).unwrap();
                cmd.render_help(self.wrap_width)
            }
            MatchError::DisplayVersion => {
                let cmd = self.last_cmd();
                cmd.render_version()
            }
            MatchError::InvalidSubcommand(arg) => {
                exit = 1;
                let cmd = self.last_cmd();
                let cmd_str = cmd.cmd_paths().join("-");
                let names = cmd.list_subcommand_names().join(", ");
                let details = match arg {
                    Some(arg) => format!("but '{arg}' is not one of them"),
                    None => "but one was not provided".to_string(),
                };
                format!(
                    r###"error: `{cmd_str}` requires a subcommand {details}
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
            MatchError::InvalidBindEnvironment(_level, value, env_name, name, choices) => {
                exit = 1;
                if choices.is_empty() {
                    format!(
                        r###"error: environment variable `{env_name}` has invalid value for param '{name}'"###
                    )
                } else {
                    let list = choices.join(", ");
                    format!(
                        r###"error: invalid value `{value}` for environment variable `{env_name}` that bound to `{name}`
  [possible values: {list}]"###
                    )
                }
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

    #[cfg(feature = "compgen")]
    fn comp_flag_options(&self, combine: bool) -> Vec<CompItem> {
        let mut output = vec![];
        let level = self.cmds.len() - 1;
        let last_cmd = self.last_cmd();
        let args: HashSet<&str> = self.flag_option_args[level]
            .iter()
            .filter_map(|v| v.2)
            .collect();
        let last = self.args.last().map(|v| v.as_str()).unwrap_or_default();
        let params = if combine {
            last_cmd.flag_option_params.iter().collect()
        } else {
            last_cmd.all_flag_options()
        };
        for param in params.iter() {
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
                    let nospace = param.prefixed() || param.assigned();
                    let v = if param.assigned() && v.len() > 2 {
                        format!("{v}=")
                    } else {
                        v
                    };
                    output.push((v, describe.to_string(), nospace, kind))
                }
            }
        }
        output
    }

    fn last_cmd(&self) -> &Command {
        self.cmds.last().unwrap()
    }
}

#[derive(Debug)]
struct BindEnvs<'a, 'x> {
    flag_options: Vec<BindEnvMap<'a, 'x>>,
    positionals: BindEnvMap<'a, 'x>,
    choice_fns: HashSet<&'a str>,
}

impl BindEnvs<'_, '_> {
    fn new(len: usize) -> Self {
        Self {
            flag_options: vec![HashMap::new(); len],
            positionals: HashMap::new(),
            choice_fns: HashSet::new(),
        }
    }
}

type BindEnvMap<'a, 'x> = HashMap<&'a str, Vec<&'x str>>;

fn find_subcommand<'a>(
    cmd: &'a Command,
    arg: &str,
    positional_args: &[&str],
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

fn take_value_args<'a>(
    args: &'a [String],
    start: usize,
    len: usize,
    signs: &IndexSet<char>,
    assigned: bool,
    compgen: bool,
) -> Vec<&'a str> {
    let mut output = vec![];
    if assigned || len == 0 {
        return output;
    }
    let args_len = args.len();
    let end = (start + len).min(args_len);
    for (i, arg) in args.iter().enumerate().take(end).skip(start) {
        if maybe_flag_option(arg, signs) && (arg.len() > 1 || (compgen && i == args_len - 1)) {
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
    signs: &IndexSet<char>,
    compgen: bool,
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
        let value_args = take_value_args(args, *arg_index + 1, 1, signs, param.assigned(), compgen);
        let take_empty = value_args.is_empty();
        *arg_index += value_args.len();
        let values = if take_empty { vec!["1"] } else { value_args };
        if !param.assigned() && *arg_index == args_len - 1 {
            *arg_comp = ArgComp::OptionValue(param.id().to_string(), 0);
            if take_empty {
                *split_last_arg_at = Some(prefix_len);
            }
        }
        flag_option_args.push((arg, values, Some(param.id())));
    } else {
        let values_max = param.num_args().1;
        let args_len = args.len();
        let value_args = take_value_args(
            args,
            *arg_index + 1,
            values_max,
            signs,
            param.assigned(),
            compgen,
        );
        *arg_index += value_args.len();
        if *arg_index == args_len - 1 {
            if *arg_comp != ArgComp::FlagOrOption {
                if param.is_option() && !param.assigned() && value_args.len() <= values_max {
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

#[cfg(feature = "compgen")]
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

#[cfg(feature = "compgen")]
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
            if (flag && maybe_flag_option(&v, &signs)) || (!flag && !maybe_flag_option(&v, &signs))
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

#[cfg(feature = "compgen")]
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

#[cfg(feature = "compgen")]
fn comp_flag_option(param: &FlagOptionParam, index: usize) -> Vec<CompItem> {
    let value_name = param
        .notations()
        .get(index)
        .map(|v| v.as_str())
        .unwrap_or_else(|| param.notations().last().unwrap());
    comp_param(param.describe_oneline(), value_name, param.data())
}

#[cfg(feature = "compgen")]
fn comp_positional(param: &PositionalParam) -> Vec<CompItem> {
    comp_param(param.describe_oneline(), param.notation(), param.data())
}

#[cfg(feature = "compgen")]
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
    choice: Option<&'a ChoiceValue>,
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

fn delimit_arg_values<'x, T: Param>(param: &T, values: &[&'x str]) -> Vec<&'x str> {
    if let Some(delimiter) = param.delimiter() {
        values.iter().flat_map(|v| v.split(delimiter)).collect()
    } else {
        values.to_vec()
    }
}

fn is_bool_value(value: &str) -> bool {
    matches!(value, "true" | "false" | "0" | "1")
}

fn maybe_flag_option(arg: &str, signs: &IndexSet<char>) -> bool {
    let cond = if signs.contains(&'+') && arg.starts_with('+') {
        !arg.starts_with("++")
    } else if arg.starts_with('-') {
        arg.len() < 3 || !arg.starts_with("---")
    } else {
        false
    };
    if !cond {
        return false;
    }
    let value = match arg.split_once('=') {
        Some((v, _)) => v,
        _ => arg,
    };
    if value.contains(|c: char| c.is_whitespace()) {
        return false;
    }
    true
}
