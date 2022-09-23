use crate::parser::{parse, Event, EventData};
use crate::Result;
use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, HashSet};

pub fn compgen(source: &str, args: &[&str]) -> Result<Vec<String>> {
    let events = parse(source)?;
    let cmd_comp = Completion::new_from_events(&events);
    cmd_comp.generate(args)
}

#[derive(Debug, Default)]
pub struct Completion {
    aliases: IndexSet<String>,
    mappings: IndexMap<String, String>,
    options: HashMap<String, (Option<String>, Vec<String>, bool)>,
    flags: HashMap<String, Option<String>>,
    positionals: IndexMap<String, Vec<String>>,
    subcommands: IndexMap<String, Completion>,
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
                        let short = if let Some(short) = option_param.short.as_ref() {
                            let short = format!("-{}", short);
                            cmd.mappings.insert(short.clone(), name.clone());
                            Some(short)
                        } else {
                            None
                        };
                        let choices = match &option_param.choices {
                            Some(choices) => choices.iter().map(|v| v.to_string()).collect(),
                            None => vec![],
                        };
                        cmd.mappings.insert(name.clone(), name.clone());
                        cmd.options
                            .insert(name.clone(), (short, choices, option_param.multiple));
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
                        let short = if let Some(short) = flag_param.short.as_ref() {
                            let short = format!("-{}", short);
                            cmd.mappings.insert(short.clone(), name.clone());
                            Some(short)
                        } else {
                            None
                        };
                        cmd.mappings.insert(name.clone(), name.clone());
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
                        let multiple = if positional_param.multiple { "..." } else { "" };
                        let choices = match &positional_param.choices {
                            Some(choices) => choices.iter().map(|v| v.to_string()).collect(),
                            None => vec![],
                        };
                        cmd.positionals.insert(
                            format!("<{}>{}", positional_param.name.to_uppercase(), multiple),
                            choices,
                        );
                    }
                }
                EventData::Func(name) => {
                    is_root_scope = false;
                    let name = name.to_string();
                    if let Some(mut cmd) = maybe_subcommand.take() {
                        root_cmd.mappings.insert(name.clone(), name.clone());
                        for alias in cmd.aliases.drain(..) {
                            root_cmd.mappings.insert(alias, name.clone());
                        }
                        root_cmd.subcommands.insert(name.clone(), cmd);
                    }
                }
                _ => {}
            }
        }
        if help_subcommand {
            let mut cmd = Completion::default();
            cmd.positionals.insert(
                "<CMD>".to_string(),
                root_cmd.subcommands.keys().map(|v| v.to_string()).collect(),
            );
            root_cmd
                .mappings
                .insert("help".to_string(), "help".to_string());
            root_cmd.subcommands.insert("help".into(), cmd);
        }
        root_cmd
    }

    pub fn generate(&self, args: &[&str]) -> Result<Vec<String>> {
        let mut i = 1;
        let len = args.len();
        let mut omitted: HashSet<String> = HashSet::default();
        let mut cmd_comp = self;
        let mut positional_index = 0;
        let mut unknown_arg = false;
        while i < len {
            let arg = args[i];
            if let Some(name) = cmd_comp.mappings.get(arg) {
                if arg.starts_with('-') {
                    if let Some((short, choices, multiple)) = cmd_comp.options.get(name) {
                        if i == len - 1 {
                            return Ok(choices.clone());
                        }
                        if *multiple {
                            while i + 1 < len && !args[i + 1].starts_with('-') {
                                i += 1;
                            }
                        } else {
                            if !args[i + 1].starts_with('-') {
                                i += 1;
                            }
                            omitted.insert(name.to_string());
                            if let Some(short) = short {
                                omitted.insert(short.to_string());
                            }
                        }
                    } else if let Some(short) = cmd_comp.flags.get(name) {
                        omitted.insert(name.to_string());
                        if let Some(short) = short {
                            omitted.insert(short.to_string());
                        }
                    }
                } else if let Some(cmd) = cmd_comp.subcommands.get(name) {
                    cmd_comp = cmd;
                    omitted.clear();
                    positional_index = 0;
                }
            } else if arg.starts_with('-') {
                unknown_arg = true;
                positional_index = 0;
            } else if !unknown_arg {
                positional_index += 1;
            }
            i += 1;
        }
        let mut output = vec![];
        for name in cmd_comp.mappings.keys() {
            if !omitted.contains(name) {
                output.push(name.to_string());
            }
        }
        if positional_index >= cmd_comp.positionals.len() {
            if let Some((name, _)) = cmd_comp.positionals.last() {
                if name.ends_with("...") {
                    output.push(name.to_string());
                }
            }
        } else if let Some((name, choices)) = cmd_comp.positionals.iter().nth(positional_index) {
            if choices.is_empty() {
                output.push(name.to_string())
            } else {
                output.extend(choices.to_vec());
            }
        }
        Ok(output)
    }
}
