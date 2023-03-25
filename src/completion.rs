use crate::parser::{parse, Event, EventData, EventScope};
use crate::utils::split_shell_words;
use crate::Result;
use anyhow::anyhow;
use either::Either;
use indexmap::{IndexMap, IndexSet};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub fn compgen(source: &str, line: &str) -> Result<Vec<String>> {
    let events = parse(source)?;
    let comp = Completion::new_from_events(&events);
    comp.generate(line)
}

type ChoicesType = Either<Vec<String>, String>;
type OptionMapType = (Option<String>, Vec<String>, Option<ChoicesType>, bool);
type PositionalItemType = (String, Option<ChoicesType>, bool);

pub(crate) const DYNAMIC_COMPGEN_FN: &str = "_compgen";

#[derive(Default)]
pub struct Completion {
    name: Option<String>,
    help: bool,
    aliases: IndexSet<String>,
    options: HashMap<String, OptionMapType>,
    flags: HashMap<String, Option<String>>,
    positionals: Vec<PositionalItemType>,
    subcommands: Vec<Completion>,
    subcommand_mappings: IndexMap<String, String>,
    flag_option_mappings: IndexMap<String, String>,
    root: Arc<RefCell<RootData>>,
}

impl Completion {
    pub fn new_from_events(events: &[Event]) -> Self {
        let mut root_comp = Completion::default();
        let root_data = root_comp.root.clone();
        for Event { data, .. } in events {
            match data {
                EventData::Help(_) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    cmd.help = true;
                }
                EventData::Cmd(_) => {
                    root_data.borrow_mut().scope = EventScope::CmdStart;
                    root_comp.create_subcommand();
                }
                EventData::Aliases(aliases) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    cmd.aliases.extend(aliases.iter().map(|v| v.to_string()))
                }
                EventData::Option(option_param) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    let name = format!("--{}", option_param.name);
                    cmd.flag_option_mappings.insert(name.clone(), name.clone());
                    let short = if let Some(short) = option_param.short.as_ref() {
                        let short = format!("-{}", short);
                        cmd.flag_option_mappings.insert(short.clone(), name.clone());
                        Some(short)
                    } else {
                        None
                    };
                    let choices =
                        parse_choices_or_fn(&option_param.choices, &option_param.choices_fn);
                    cmd.options.insert(
                        name.clone(),
                        (
                            short,
                            option_param.arg_value_names.clone(),
                            choices,
                            option_param.multiple,
                        ),
                    );
                }
                EventData::Flag(flag_param) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    let name = format!("--{}", flag_param.name);
                    cmd.flag_option_mappings.insert(name.clone(), name.clone());
                    let short = if let Some(short) = flag_param.short.as_ref() {
                        let short = format!("-{}", short);
                        cmd.flag_option_mappings.insert(short.clone(), name.clone());
                        Some(short)
                    } else {
                        None
                    };
                    cmd.flags.insert(name, short);
                }
                EventData::Positional(positional_param) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    let choices = parse_choices_or_fn(
                        &positional_param.choices,
                        &positional_param.choices_fn,
                    );
                    cmd.positionals.push((
                        positional_param.arg_value_name.to_string(),
                        choices,
                        positional_param.multiple,
                    ));
                }
                EventData::Func(name) => {
                    if root_data.borrow().scope == EventScope::CmdStart {
                        let (parent, child) = match name.split_once("::") {
                            None => (name.as_str(), None),
                            Some((parent, child)) => (parent, Some(child)),
                        };
                        match child {
                            None => {
                                let comp = root_comp.subcommands.last_mut().unwrap();
                                comp.name = Some(name.to_string());
                                root_comp
                                    .subcommand_mappings
                                    .insert(name.to_string(), name.to_string());
                                for alias in comp.aliases.iter() {
                                    root_comp
                                        .subcommand_mappings
                                        .insert(alias.to_string(), name.to_string());
                                }
                            }
                            Some(child) => {
                                let mut comp = root_comp.subcommands.pop().unwrap();
                                comp.name = Some(child.to_string());
                                if let Some(parent_comp) = root_comp
                                    .subcommands
                                    .iter_mut()
                                    .find(|v| v.name == Some(parent.into()))
                                {
                                    parent_comp
                                        .subcommand_mappings
                                        .insert(child.to_string(), child.to_string());
                                    for alias in comp.aliases.iter() {
                                        parent_comp
                                            .subcommand_mappings
                                            .insert(alias.to_string(), child.to_string());
                                    }
                                    parent_comp.subcommands.push(comp);
                                }
                            }
                        }
                        root_data.borrow_mut().scope = EventScope::FnEnd;
                    } else if name == DYNAMIC_COMPGEN_FN {
                        root_data.borrow_mut().dynamic_compgen = true;
                    }
                }
                _ => {}
            }
        }
        root_comp.add_help_subcommand();
        root_comp
    }

    pub fn generate(&self, line: &str) -> Result<Vec<String>> {
        let mut args = split_shell_words(line).map_err(|_| anyhow!("Invalid args"))?;
        if line
            .chars()
            .last()
            .map(|v| v.is_ascii_whitespace())
            .unwrap_or(false)
        {
            args.push(" ".into());
        }
        let mut comp_type = get_comp_type(&args);
        let mut force_positional = false;
        let mut option_complete_values: Option<Vec<String>> = None;
        let mut index = 0;
        let mut skipped: HashSet<String> = HashSet::default();
        let mut parent_comp = self;
        let mut comp = self;
        let mut positional_index = 0;
        let mut has_subcommand = false;
        let len = args.len();
        while index < len {
            let current_arg = args[index].as_str();
            if current_arg == "--" {
                force_positional = true;
            } else if current_arg.starts_with('-') && !force_positional {
                let (arg_name, arg_has_value) = match current_arg.split_once('=') {
                    Some((v, _)) => (v, true),
                    None => (current_arg, false),
                };
                if let Some(name) = comp.flag_option_mappings.get(arg_name) {
                    if let Some((short, value_names, choices, multiple)) = comp.options.get(name) {
                        if !multiple {
                            skipped.insert(name.to_string());
                            if let Some(short) = short {
                                skipped.insert(short.to_string());
                            }
                        }
                        if !arg_has_value {
                            let mut value_name = None;
                            let mut i = 0;
                            loop {
                                match (value_names.get(i), args.get(index + 1 + i)) {
                                    (Some(_), Some(arg)) => {
                                        if is_flag_or_option(arg) {
                                            index += i;
                                            break;
                                        }
                                    }
                                    (None, Some(_)) => {
                                        index += i;
                                        break;
                                    }
                                    (_, None) => {
                                        if i > 0 {
                                            value_name = value_names.get(i - 1);
                                        }
                                        break;
                                    }
                                }
                                i += 1;
                            }
                            if let Some(value_name) = value_name {
                                comp_type = CompType::OptionValue;
                                option_complete_values = Some(generate_by_choices_or_name(
                                    value_name, choices, *multiple,
                                ))
                            }
                        }
                    } else if let Some(short) = comp.flags.get(name) {
                        skipped.insert(name.to_string());
                        if let Some(short) = short {
                            skipped.insert(short.to_string());
                        }
                    }
                } else if let (Some(next), Some(next2)) = (args.get(index + 1), args.get(index + 2))
                {
                    if !is_flag_or_option(next) && is_flag_or_option(next2) {
                        index += 1;
                    }
                }
            } else if !current_arg.starts_with('-') {
                let mut matched = false;
                if positional_index == 0 {
                    if let Some(name) = comp.subcommand_mappings.get(current_arg) {
                        if let Some(subcmd_comp) = comp
                            .subcommands
                            .iter()
                            .find(|v| v.name == Some(name.into()))
                        {
                            skipped.clear();
                            has_subcommand = true;
                            parent_comp = comp;
                            comp = subcmd_comp;
                            matched = true;
                            skipped.insert(name.to_string());
                            skipped.extend(subcmd_comp.aliases.iter().cloned());
                        }
                    }
                }
                if !matched {
                    positional_index += 1;
                }
            } else {
                positional_index += 1;
            }
            index += 1;
        }
        if positional_index == 1 && comp_type == CompType::Any {
            positional_index = 0
        }
        let mut output = vec![];
        match comp_type {
            CompType::FlagOrOption => {
                add_mapping_to_output(&mut output, &skipped, &comp.flag_option_mappings);
            }
            CompType::CommandOrPositional => {
                if has_subcommand {
                    if positional_index == 0 {
                        add_mapping_to_output(
                            &mut output,
                            &skipped,
                            &parent_comp.subcommand_mappings,
                        );
                    } else {
                        add_positional_to_output(
                            &mut output,
                            positional_index - 1,
                            &comp.positionals,
                        );
                    }
                } else {
                    add_mapping_to_output(&mut output, &skipped, &parent_comp.subcommand_mappings);
                    add_positional_to_output(
                        &mut output,
                        positional_index.saturating_sub(1),
                        &comp.positionals,
                    );
                }
            }
            CompType::OptionValue => {
                if let Some(values) = option_complete_values {
                    output.extend(values)
                }
            }
            CompType::Any => {
                add_mapping_to_output(&mut output, &skipped, &comp.flag_option_mappings);
                if positional_index == 0 {
                    add_mapping_to_output(&mut output, &skipped, &comp.subcommand_mappings);
                }
                add_positional_to_output(&mut output, positional_index, &comp.positionals);
            }
        }
        if self.root.borrow().dynamic_compgen && output.iter().all(|v| !v.starts_with('`')) {
            output.push(format!("`{}`", DYNAMIC_COMPGEN_FN));
        }
        Ok(output)
    }

    fn get_cmd(comp: &mut Self) -> &mut Self {
        if comp.subcommands.last().is_some() {
            comp.subcommands.last_mut().unwrap()
        } else {
            comp
        }
    }

    fn create_subcommand(&mut self) {
        let comp = Completion::default();
        self.subcommands.push(comp);
    }

    fn add_help_subcommand(&mut self) {
        if self.help {
            let mut help_comp = Completion {
                name: Some("help".into()),
                ..Default::default()
            };
            let mut help_choices = vec![];
            for subcmd in self.subcommands.iter_mut() {
                subcmd.add_help_subcommand();
                if let Some(name) = &subcmd.name {
                    help_choices.push(name.to_string());
                }
            }
            help_comp.positionals.push((
                "<CMD>".to_string(),
                Some(Either::Left(help_choices)),
                false,
            ));
            self.subcommand_mappings
                .insert("help".to_string(), "help".to_string());
            self.subcommands.push(help_comp);
        }
    }
}

fn add_positional_to_output(
    output: &mut Vec<String>,
    positional_index: usize,
    positionals: &[PositionalItemType],
) {
    let positional_len = positionals.len();
    if positional_index >= positional_len {
        if let Some((name, choices, multiple)) = positionals.last() {
            if *multiple {
                output.extend(generate_by_choices_or_name(name, choices, *multiple));
            }
        }
    } else if let Some((name, choices, multiple)) = positionals.get(positional_index) {
        output.extend(generate_by_choices_or_name(name, choices, *multiple));
    }
}

#[derive(Default)]
struct RootData {
    scope: EventScope,
    dynamic_compgen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompType {
    FlagOrOption,
    CommandOrPositional,
    OptionValue,
    Any,
}

fn get_comp_type(args: &[String]) -> CompType {
    if let Some(arg) = args.last() {
        if arg.starts_with('-') {
            CompType::FlagOrOption
        } else if arg.as_str() == " " {
            CompType::Any
        } else {
            CompType::CommandOrPositional
        }
    } else {
        CompType::Any
    }
}

fn parse_choices_or_fn(
    choices: &Option<Vec<String>>,
    choices_fn: &Option<String>,
) -> Option<ChoicesType> {
    if let Some(choices_fn) = choices_fn {
        Some(Either::Right(format!("`{}`", choices_fn)))
    } else {
        choices
            .as_ref()
            .map(|choices| Either::Left(choices.iter().map(|v| v.to_string()).collect()))
    }
}

fn generate_by_choices_or_name(
    value_name: &str,
    choices: &Option<ChoicesType>,
    multiple: bool,
) -> Vec<String> {
    if let Some(choices) = choices {
        match choices {
            Either::Left(choices) => choices.to_vec(),
            Either::Right(choices_fn) => vec![choices_fn.to_string()],
        }
    } else {
        let value = if multiple {
            format!("<{}>...", value_name)
        } else {
            format!("<{}>", value_name)
        };
        vec![value]
    }
}

fn add_mapping_to_output(
    output: &mut Vec<String>,
    skipped: &HashSet<String>,
    mapping: &IndexMap<String, String>,
) {
    for name in mapping.keys() {
        if !skipped.contains(name) {
            output.push(name.to_string());
        }
    }
}

fn is_flag_or_option(arg: &str) -> bool {
    arg != "--" && arg.starts_with('-')
}
