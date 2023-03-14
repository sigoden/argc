use crate::parser::{parse, Event, EventData};
use crate::utils::split_shell_words;
use crate::Result;
use anyhow::anyhow;
use either::Either;
use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, HashSet};

pub fn compgen(source: &str, line: &str) -> Result<Vec<String>> {
    let events = parse(source)?;
    let comp = Completion::new_from_events(&events);
    comp.generate(line)
}

type ChoicesType = Either<Vec<String>, String>;
type OptionMapType = (Option<String>, String, Option<ChoicesType>, bool);
type PositionalItemType = (String, Option<ChoicesType>, bool);

#[derive(Debug, Default)]
pub struct Completion {
    aliases: IndexSet<String>,
    options: HashMap<String, OptionMapType>,
    flags: HashMap<String, Option<String>>,
    flag_option_mappings: IndexMap<String, String>,
    positionals: Vec<PositionalItemType>,
    command_mappings: IndexMap<String, String>,
    commands: IndexMap<String, Completion>,
}

impl Completion {
    pub fn new_from_events(events: &[Event]) -> Self {
        let mut root_cmd = Completion::default();
        let mut maybe_subcommand: Option<Completion> = None;
        let mut is_root_scope = true;
        let mut help_subcommand = false;
        for Event { data, .. } in events {
            match data {
                EventData::Help(_) => {
                    help_subcommand = true;
                }
                EventData::Cmd(_) => {
                    is_root_scope = false;
                    maybe_subcommand = Some(Completion::default())
                }
                EventData::Aliases(aliases) => {
                    if let Some(cmd) = &mut maybe_subcommand {
                        cmd.aliases.extend(aliases.iter().map(|v| v.to_string()))
                    }
                }
                EventData::Option(option_param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    if let Some(cmd) = cmd {
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
                                option_param.arg_value_name.clone(),
                                choices,
                                option_param.multiple,
                            ),
                        );
                    }
                }
                EventData::Flag(flag_param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    if let Some(cmd) = cmd {
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
                }
                EventData::Positional(positional_param) => {
                    let cmd = maybe_subcommand.as_mut().or(if is_root_scope {
                        Some(&mut root_cmd)
                    } else {
                        None
                    });
                    if let Some(cmd) = cmd {
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
                }
                EventData::Func(name) => {
                    is_root_scope = false;
                    let name = name.to_string();
                    if let Some(cmd) = maybe_subcommand.take() {
                        root_cmd.command_mappings.insert(name.clone(), name.clone());
                        for alias in cmd.aliases.iter() {
                            root_cmd
                                .command_mappings
                                .insert(alias.to_string(), name.clone());
                        }
                        root_cmd.commands.insert(name.clone(), cmd);
                    }
                }
                _ => {}
            }
        }
        if help_subcommand {
            let mut cmd = Completion::default();
            cmd.positionals.push((
                "<CMD>".to_string(),
                Some(Either::Left(
                    root_cmd.commands.keys().map(|v| v.to_string()).collect(),
                )),
                false,
            ));
            root_cmd
                .command_mappings
                .insert("help".to_string(), "help".to_string());
            root_cmd.commands.insert("help".into(), cmd);
        }
        root_cmd
    }

    pub fn generate(&self, line: &str) -> Result<Vec<String>> {
        let args = split_shell_words(line).map_err(|_| anyhow!("Invalid args"))?;
        let mut comp_type = get_comp_type(line, &args);
        let mut force_positional = false;
        let mut option_complete_values = None;
        let mut index = 0;
        let mut skipped: HashSet<String> = HashSet::default();
        let root_comp = self;
        let mut comp = self;
        let mut positional_index = 0;
        let mut has_subcommand = false;
        let len = args.len();
        while index < len {
            let arg = args[index].as_str();
            if arg == "--" {
                force_positional = true;
            } else if arg.starts_with('-') && !force_positional {
                let (arg, arg_has_value) = match arg.split_once('=') {
                    Some((v, _)) => (v, true),
                    None => (arg, false),
                };
                if let Some(name) = comp.flag_option_mappings.get(arg) {
                    if let Some((short, value_name, choices, multiple)) = comp.options.get(name) {
                        if !multiple {
                            skipped.insert(name.to_string());
                            if let Some(short) = short {
                                skipped.insert(short.to_string());
                            }
                        }
                        if index == len - 1 {
                            if !arg_has_value && comp_type == CompType::Any {
                                comp_type = CompType::OptionValue;
                                option_complete_values =
                                    Some(generate_by_choices_or_name(value_name, choices))
                            }
                            break;
                        }
                        if !arg_has_value && !args[index + 1].starts_with('-') {
                            index += 1;
                            if index == len - 1 && comp_type == CompType::CommandOrPositional {
                                comp_type = CompType::OptionValue;
                                option_complete_values =
                                    Some(generate_by_choices_or_name(value_name, choices));
                                break;
                            }
                        }
                    } else if let Some(short) = comp.flags.get(name) {
                        skipped.insert(name.to_string());
                        if let Some(short) = short {
                            skipped.insert(short.to_string());
                        }
                    }
                }
            } else if !arg.starts_with('-') {
                let mut matched = false;
                if positional_index == 0 {
                    if let Some(name) = comp.command_mappings.get(arg) {
                        if let Some(cmd) = comp.commands.get(name) {
                            skipped.clear();
                            has_subcommand = true;
                            comp = cmd;
                            matched = true;
                            skipped.insert(name.to_string());
                            skipped.extend(cmd.aliases.iter().cloned());
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
        let mut output = vec![];
        match comp_type {
            CompType::FlagOrOption => {
                add_mapping_to_output(&mut output, &skipped, &comp.flag_option_mappings);
            }
            CompType::CommandOrPositional => {
                if has_subcommand {
                    if positional_index == 0 {
                        add_mapping_to_output(&mut output, &skipped, &root_comp.command_mappings);
                    } else {
                        add_positional_to_output(
                            &mut output,
                            positional_index - 1,
                            &comp.positionals,
                        );
                    }
                } else {
                    add_mapping_to_output(&mut output, &skipped, &root_comp.command_mappings);
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
                    add_mapping_to_output(&mut output, &skipped, &comp.command_mappings);
                }
                add_positional_to_output(&mut output, positional_index, &comp.positionals);
            }
        }
        Ok(output)
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
                output.extend(generate_by_choices_or_name(name, choices));
            }
        }
    } else if let Some((name, choices, _)) = positionals.get(positional_index) {
        output.extend(generate_by_choices_or_name(name, choices));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompType {
    FlagOrOption,
    CommandOrPositional,
    OptionValue,
    Any,
}

fn get_comp_type(line: &str, args: &[String]) -> CompType {
    if line
        .chars()
        .last()
        .map(|v| v.is_ascii_whitespace())
        .unwrap_or(true)
    {
        CompType::Any
    } else if let Some(arg) = args.last() {
        if arg.starts_with('-') {
            CompType::FlagOrOption
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

fn generate_by_choices_or_name(value_name: &str, choices: &Option<ChoicesType>) -> Vec<String> {
    if let Some(choices) = choices {
        match choices {
            Either::Left(choices) => choices.to_vec(),
            Either::Right(choices_fn) => vec![choices_fn.to_string()],
        }
    } else {
        vec![format!("<{}>", value_name)]
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
