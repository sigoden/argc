use crate::parser::{parse, Event, EventData, EventScope};
use crate::utils::{escape_shell_words, get_shell_path, split_shell_words};
use crate::Result;
use anyhow::{anyhow, bail};
use either::Either;
use indexmap::{IndexMap, IndexSet};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use std::{process, str::FromStr};

pub fn compgen(
    shell: Shell,
    script_path: &str,
    script_content: &str,
    line: &str,
) -> Result<String> {
    let (last_word, unbalance_char) = get_last_word(line);
    let line = if let Some(c) = unbalance_char {
        format!("{}{}", line, c)
    } else {
        line.to_string()
    };
    let candicates = generate_candiates(script_content, &line)?;
    let candicates = expand_candicates(candicates, script_path, &line, &last_word)?;
    shell.convert(&candicates, &last_word)
}

fn generate_candiates(source: &str, line: &str) -> Result<Vec<(String, String)>> {
    let events = parse(source)?;
    let comp = Completion::new_from_events(&events);
    comp.generate(line)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Powershell,
    Fish,
}

impl FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "powershell" => Ok(Self::Powershell),
            "fish" => Ok(Self::Fish),
            _ => bail!("Invalid shell value, must be one of {}", Shell::list()),
        }
    }
}

impl Shell {
    pub fn list() -> &'static str {
        "bash,zsh,powershell,fish"
    }
    pub fn convert(&self, candicates: &[(String, String)], last_word: &str) -> Result<String> {
        let no_description = compgen_no_description();
        if candicates.len() == 1 {
            return Ok(self.convert_value(&candicates[0].0, last_word));
        }
        let output = candicates
            .iter()
            .map(|(value, description)| {
                if no_description {
                    self.convert_candiate(value, last_word, "")
                } else {
                    self.convert_candiate(value, last_word, description)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        Ok(output)
    }

    pub fn convert_value(&self, value: &str, last_word: &str) -> String {
        if value.starts_with("__argc_") {
            if value.starts_with("__argc_value") {
                return convert_arg_value(value);
            } else {
                return value.to_string();
            }
        }
        match self {
            Shell::Bash => {
                if last_word.contains(':') {
                    if let Some((prefix, _)) = last_word.rsplit_once(':') {
                        if let Some(value) = value.strip_prefix(&last_word[0..prefix.len() + 1]) {
                            return value.to_string();
                        }
                    }
                }
                bash_escape(value)
            }
            Shell::Zsh => zsh_escape(value),
            Shell::Powershell => format!("{} ", powershell_escape(value)),
            Shell::Fish => value.to_string(),
        }
    }

    pub fn convert_candiate(&self, value: &str, last_word: &str, description: &str) -> String {
        let value = self.convert_value(value, last_word);
        if description.is_empty() {
            value
        } else {
            match self {
                Shell::Bash => value,
                Shell::Zsh => format!("{}:{}", value, description),
                Shell::Powershell => format!("{}\t{}", value, description),
                Shell::Fish => format!("{}\t{}", value, description),
            }
        }
    }
}

type ChoicesValue = Either<Vec<String>, String>;

struct OptionValue {
    summary: String,
    short: Option<String>,
    value_names: Vec<String>,
    multiple: bool,
    choices: Option<ChoicesValue>,
}

struct FlagValue {
    summary: String,
    short: Option<String>,
    multiple: bool,
}

struct PositionalValue {
    summary: String,
    value_name: String,
    choices: Option<ChoicesValue>,
    multiple: bool,
    required: bool,
}

#[derive(Default)]
struct Completion {
    name: Option<String>,
    description: String,
    aliases: IndexSet<String>,
    options: HashMap<String, OptionValue>,
    option_mappings: IndexMap<String, String>,
    flags: HashMap<String, FlagValue>,
    flag_mappings: IndexMap<String, String>,
    positionals: Vec<PositionalValue>,
    subcommands: Vec<Completion>,
    subcommand_mappings: IndexMap<String, String>,
    root: Arc<RefCell<RootData>>,
}

impl Completion {
    pub fn new_from_events(events: &[Event]) -> Self {
        let mut root_comp = Completion::default();
        let root_data = root_comp.root.clone();
        for Event { data, .. } in events {
            match data {
                EventData::Cmd(value) => {
                    root_data.borrow_mut().scope = EventScope::CmdStart;
                    let cmd = root_comp.create_subcommand();
                    cmd.description = value.to_string();
                }
                EventData::Aliases(aliases) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    cmd.aliases.extend(aliases.iter().map(|v| v.to_string()))
                }
                EventData::Option(option_param) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    let name = format!("--{}", option_param.name);
                    cmd.option_mappings.insert(name.clone(), name.clone());
                    let short = if let Some(short) = option_param.short.as_ref() {
                        let short = format!("-{}", short);
                        cmd.option_mappings.insert(short.clone(), name.clone());
                        Some(short)
                    } else {
                        None
                    };
                    let choices =
                        parse_choices_or_fn(&option_param.choices, &option_param.choices_fn);
                    cmd.options.insert(
                        name.clone(),
                        OptionValue {
                            short,
                            summary: option_param.summary.clone(),
                            value_names: option_param.arg_value_names.clone(),
                            choices,
                            multiple: option_param.multiple,
                        },
                    );
                }
                EventData::Flag(flag_param) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    let name = format!("--{}", flag_param.name);
                    cmd.flag_mappings.insert(name.clone(), name.clone());
                    let short = if let Some(short) = flag_param.short.as_ref() {
                        let short = format!("-{}", short);
                        cmd.flag_mappings.insert(short.clone(), name.clone());
                        Some(short)
                    } else {
                        None
                    };
                    cmd.flags.insert(
                        name,
                        FlagValue {
                            summary: flag_param.summary.clone(),
                            short,
                            multiple: flag_param.multiple,
                        },
                    );
                }
                EventData::Positional(positional_param) => {
                    let cmd = Self::get_cmd(&mut root_comp);
                    let choices = parse_choices_or_fn(
                        &positional_param.choices,
                        &positional_param.choices_fn,
                    );
                    cmd.positionals.push(PositionalValue {
                        summary: positional_param.summary.clone(),
                        value_name: positional_param.arg_value_name.to_string(),
                        choices,
                        multiple: positional_param.multiple,
                        required: positional_param.required,
                    });
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
                    }
                }
                _ => {}
            }
        }
        root_comp.add_help_subcommand();
        root_comp
    }

    pub fn generate(&self, line: &str) -> Result<Vec<(String, String)>> {
        let args = split_shell_words(line).map_err(|_| anyhow!("Invalid args"))?;
        let mut comp_type = get_comp_type(line, &args);
        let mut force_positional = false;
        let mut option_complete_values: Option<Vec<(String, String)>> = None;
        let mut index = 0;
        let mut skipped_flags_options: HashSet<String> = HashSet::default();
        let mut parent_comp = self;
        let mut comp = self;
        let mut positional_index: usize = 0;
        let mut subcommand_name = None;
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
                if let Some(name) = comp.option_mappings.get(arg_name) {
                    if let Some(option) = comp.options.get(name) {
                        if !option.multiple {
                            skipped_flags_options.insert(name.to_string());
                            if let Some(short) = &option.short {
                                skipped_flags_options.insert(short.to_string());
                            }
                        }
                        if !arg_has_value {
                            let mut value_name = None;
                            let mut i = 0;
                            loop {
                                match (option.value_names.get(i), args.get(index + 1 + i)) {
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
                                    (maybe_value_name, None) => {
                                        if comp_type == CompType::Any && maybe_value_name.is_some()
                                        {
                                            value_name = option.value_names.get(i);
                                        } else if comp_type == CompType::CommandOrPositional
                                            && i > 0
                                        {
                                            value_name = option.value_names.get(i - 1);
                                        }
                                        index += i;
                                        break;
                                    }
                                }
                                i += 1;
                            }
                            if let Some(value_name) = value_name {
                                comp_type = CompType::OptionValue;
                                option_complete_values = Some(generate_by_choices_or_name(
                                    &option.summary,
                                    value_name,
                                    &option.choices,
                                    option.multiple,
                                    true,
                                ))
                            }
                        }
                    }
                } else if let Some(name) = comp.flag_mappings.get(arg_name) {
                    if let Some(flag) = comp.flags.get(name) {
                        if !flag.multiple {
                            skipped_flags_options.insert(name.to_string());
                            if let Some(short) = &flag.short {
                                skipped_flags_options.insert(short.to_string());
                            }
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
                            skipped_flags_options.clear();
                            subcommand_name = Some(name.to_string());
                            parent_comp = comp;
                            comp = subcmd_comp;
                            matched = true;
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
                comp.output_flags_and_options(&mut output, &skipped_flags_options);
            }
            CompType::CommandOrPositional if subcommand_name == Some("help".into()) => {
                parent_comp.output_subcommands(&mut output);
            }
            CompType::CommandOrPositional => {
                if subcommand_name.is_some() && positional_index == 0 {
                    parent_comp.output_subcommands(&mut output);
                    output_positionals(&mut output, 0, &parent_comp.positionals);
                } else {
                    if positional_index == 1 {
                        comp.output_subcommands(&mut output);
                    }
                    output_positionals(
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
            CompType::Any if subcommand_name == Some("help".into()) => {
                parent_comp.output_subcommands(&mut output);
            }
            CompType::Any => {
                if positional_index == 0 {
                    comp.output_subcommands(&mut output);
                }
                output_positionals(&mut output, positional_index, &comp.positionals);
                let mut has_flags_and_options = !force_positional;
                if output.len() > 1
                    || (output.len() == 1
                        && !(output[0].0.starts_with("__argc_value*")
                            || output[0].0.starts_with("__argc_value:")))
                {
                    has_flags_and_options = false;
                }
                let mut options = vec![];
                if has_flags_and_options {
                    comp.output_flags_and_options(&mut output, &skipped_flags_options);
                }
                options.extend(output);
                output = options;
            }
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

    fn create_subcommand(&mut self) -> &mut Self {
        let comp = Completion::default();
        self.subcommands.push(comp);
        self.subcommands.last_mut().unwrap()
    }

    fn add_help_subcommand(&mut self) {
        if !self.subcommands.is_empty() {
            let help_comp = Completion {
                name: Some("help".into()),
                ..Default::default()
            };
            self.subcommand_mappings
                .insert("help".to_string(), "help".to_string());
            for subcmd in self.subcommands.iter_mut() {
                subcmd.add_help_subcommand();
            }
            self.subcommands.push(help_comp);
        }
    }

    fn output_flags_and_options(
        &self,
        output: &mut Vec<(String, String)>,
        skipped: &HashSet<String>,
    ) {
        for (name, flag_name) in &self.flag_mappings {
            if !skipped.contains(name) {
                let summary = self
                    .flags
                    .get(flag_name)
                    .map(|v| v.summary.clone())
                    .unwrap_or_default();
                output.push((name.into(), summary));
            }
        }
        for (name, option_name) in &self.option_mappings {
            if !skipped.contains(name) {
                let summary = self
                    .options
                    .get(option_name)
                    .map(|v| v.summary.clone())
                    .unwrap_or_default();
                output.push((name.into(), summary));
            }
        }
    }

    fn output_subcommands(&self, output: &mut Vec<(String, String)>) {
        for (name, cmd_name) in &self.subcommand_mappings {
            let summary = self
                .subcommands
                .iter()
                .find(|v| v.name.as_ref() == Some(cmd_name))
                .map(|v| v.description.clone())
                .unwrap_or_default();
            output.push((name.into(), summary));
        }
    }
}

fn output_positionals(
    output: &mut Vec<(String, String)>,
    positional_index: usize,
    positionals: &[PositionalValue],
) {
    let positional_len = positionals.len();
    if positional_index >= positional_len {
        if let Some(positional) = positionals.last() {
            if positional.multiple {
                output.extend(generate_by_choices_or_name(
                    &positional.summary,
                    &positional.value_name,
                    &positional.choices,
                    positional.multiple,
                    positional.required,
                ));
            }
        }
    } else if let Some(positional) = positionals.get(positional_index) {
        output.extend(generate_by_choices_or_name(
            &positional.summary,
            &positional.value_name,
            &positional.choices,
            positional.multiple,
            positional.required,
        ));
    }
}

#[derive(Default)]
struct RootData {
    scope: EventScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompType {
    FlagOrOption,
    CommandOrPositional,
    OptionValue,
    Any,
}

fn get_comp_type(line: &str, args: &[String]) -> CompType {
    if line.is_empty() {}
    if let Some(ch) = line.chars().last() {
        if ch.is_ascii_whitespace() {
            return CompType::Any;
        }
        if let Some(arg) = args.last() {
            if arg.starts_with('-') {
                return CompType::FlagOrOption;
            }
            return CompType::CommandOrPositional;
        }
    }
    CompType::Any
}

fn parse_choices_or_fn(
    choices: &Option<Vec<String>>,
    choices_fn: &Option<String>,
) -> Option<ChoicesValue> {
    if let Some(choices_fn) = choices_fn {
        Some(Either::Right(choices_fn.to_string()))
    } else {
        choices
            .as_ref()
            .map(|choices| Either::Left(choices.iter().map(|v| v.to_string()).collect()))
    }
}

fn generate_by_choices_or_name(
    summary: &str,
    value_name: &str,
    choices: &Option<ChoicesValue>,
    multiple: bool,
    required: bool,
) -> Vec<(String, String)> {
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
        vec![(value, summary.into())]
    }
}

fn is_flag_or_option(arg: &str) -> bool {
    arg != "--" && arg.starts_with('-')
}

fn expand_candicates(
    values: Vec<(String, String)>,
    script_file: &str,
    line: &str,
    filter: &str,
) -> Result<Vec<(String, String)>> {
    let mut output = vec![];
    let mut param_fns = vec![];
    for (value, describe) in values {
        if let Some(param_fn) = value.strip_prefix("__argc_fn:") {
            param_fns.push(param_fn.to_string());
        } else if value.starts_with("__argc_") || value.starts_with(filter) {
            output.push((value, describe));
        }
    }
    if !param_fns.is_empty() {
        if let Some(shell) = get_shell_path() {
            for param_fn in param_fns {
                if let Ok(fn_output) = process::Command::new(&shell)
                    .arg(script_file)
                    .arg(&param_fn)
                    .arg(line)
                    .output()
                {
                    let fn_output = String::from_utf8_lossy(&fn_output.stdout);
                    for fn_output_line in fn_output.split('\n') {
                        let output_line = fn_output_line.trim();
                        if !output_line.is_empty()
                            && (output_line.starts_with("__argc_")
                                || output_line.starts_with(filter))
                        {
                            if let Some((x, y)) = output_line.split_once('\t') {
                                output.push((x.to_string(), y.to_string()));
                            } else {
                                output.push((output_line.to_string(), String::new()));
                            }
                        }
                    }
                }
            }
        }
    }
    if output.len() == 1 {
        let value = &output[0].0;
        if let Some(value_name) = value.strip_prefix("__argc_value") {
            if value_name.contains("PATH") || value_name.contains("FILE") {
                output[0] = ("__argc_comp:file".into(), String::new());
            } else if value_name.contains("DIR") || value_name.contains("FOLDER") {
                output[0] = ("__argc_comp:dir".into(), String::new());
            } else {
                output.clear();
            };
        }
    }
    Ok(output)
}

fn get_last_word(line: &str) -> (String, Option<char>) {
    let mut word = vec![];
    let mut balances = vec![];
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' {
            if i < chars.len() - 1 {
                i += 1;
                word.push(chars[i]);
            }
        } else if c.is_ascii_whitespace() {
            if balances.is_empty() {
                word.clear();
            } else {
                word.push(c);
            }
        } else if c == '\'' || c == '"' {
            if balances.last() == Some(&c) {
                balances.pop();
            } else {
                balances.push(c);
            }
            word.push(c);
        } else {
            word.push(c);
        }
        i += 1
    }
    if word.is_empty() {
        return (String::new(), None);
    }
    if balances.is_empty() {
        if word[0] == '\'' || word[0] == '\"' {
            return (word[1..word.len() - 1].iter().collect(), None);
        }
        return (word.into_iter().collect(), None);
    }
    (word[1..].iter().collect(), Some(word[0]))
}

fn zsh_escape(value: &str) -> String {
    value
        .chars()
        .map(|v| {
            if v == ':' {
                format!("\\{v}")
            } else {
                v.to_string()
            }
        })
        .collect::<String>()
}

fn bash_escape(value: &str) -> String {
    value
        .chars()
        .map(|v| {
            if matches!(
                v,
                ' ' | '!'
                    | '"'
                    | '$'
                    | '&'
                    | '\''
                    | '<'
                    | '>'
                    | '`'
                    | '|'
                    | '{'
                    | '}'
                    | '['
                    | ']'
                    | '^'
                    | '~'
                    | '#'
                    | '*'
                    | '?'
            ) {
                format!("\\{v}")
            } else {
                v.to_string()
            }
        })
        .collect::<String>()
}

fn powershell_escape(value: &str) -> String {
    escape_shell_words(value)
}

fn compgen_no_description() -> bool {
    match env::var("ARGC_COMPGEN_NO_DESCRIPTION") {
        Ok(v) => v == "true" || v == "1",
        Err(_) => false,
    }
}

fn convert_arg_value(name: &str) -> String {
    if let Some(value_name) = name.strip_prefix("__argc_value") {
        let (mark, value) = value_name.split_at(1);
        match mark {
            "+" => format!("<{value}>..."),
            "*" => format!("[{value}]..."),
            "!" => format!("<{value}>"),
            ":" => format!("[{value}]"),
            _ => name.to_string(),
        }
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_last_word() {
        assert_eq!(get_last_word("").0, "");
        assert_eq!(get_last_word(" ").0, "");
        assert_eq!(get_last_word("foo").0, "foo");
        assert_eq!(get_last_word("foo ").0, "");
        assert_eq!(get_last_word(" foo").0, "foo");
        assert_eq!(get_last_word("'foo'").0, "foo");
        assert_eq!(get_last_word("\"foo\"").0, "foo");
        assert_eq!(get_last_word("'abc "), ("abc ".into(), Some('\'')));
        assert_eq!(get_last_word("\"abc "), ("abc ".into(), Some('"')));
        assert_eq!(get_last_word("foo\\ def").0, "foo def");
    }
}
