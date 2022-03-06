use crate::parser::{parse, ArgData, ArgKind, Event, EventData, Position};
use crate::Result;
use anyhow::bail;
use clap::{Arg, ArgMatches, Command};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::ops::Deref;

const VARIABLE_PREFIX: &str = env!("CARGO_CRATE_NAME");

const ENTRYPOINT: &str = "main";

pub struct Runner<'a> {
    source: &'a str,
    eval: bool,
}

impl<'a> Runner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            eval: false,
        }
    }
    pub fn set_eval(mut self, eval: bool) -> Self {
        self.eval = eval;
        self
    }
    pub fn run(&self, args: &[&'a str]) -> Result<std::result::Result<String, String>> {
        let events = parse(self.source)?;
        let cmd = Cmd::create(&events)?;
        let name = args[0];
        let command = cmd.build(name)?;
        let res = command.try_get_matches_from(args);
        match res {
            Ok(matches) => Ok(Ok(cmd.retrieve(&matches, self))),
            Err(err) => Ok(Err(err.to_string())),
        }
    }
}

/// Run script with arguments, returns (stdout, stderr)
pub fn run<'a>(source: &'a str, args: &[&'a str]) -> Result<std::result::Result<String, String>> {
    let runner = Runner::new(source);
    runner.run(args)
}

#[derive(Debug, Default)]
struct Cmd<'a> {
    name: Option<(&'a str, String)>,
    describe: Option<&'a str>,
    positional_idx: usize,
    args: Vec<WrapArgData<'a>>,
    cmds: Vec<Cmd<'a>>,
    // for conflict detecting
    names: (HashMap<&'a str, Position>, HashMap<char, Position>),
    // root only props
    root: Option<RootData<'a>>,
    aliases: Vec<&'a str>,
}

#[derive(Debug, Default)]
struct RootData<'a> {
    author: Option<&'a str>,
    version: Option<&'a str>,
    names: HashMap<&'a str, Position>,
    main: bool,
}

impl<'a> Cmd<'a> {
    fn create(events: &'a [Event]) -> Result<Self> {
        let mut rootcmd = Cmd::default();
        let mut rootdata = RootData::default();
        let mut is_root_scope = true;
        let mut maybe_subcmd: Option<Cmd> = None;
        for Event { data, position } in events {
            match data {
                EventData::Describe(value) => {
                    if is_root_scope {
                        rootcmd.describe = Some(*value);
                    }
                }
                EventData::Version(value) => {
                    if is_root_scope {
                        rootdata.version = Some(*value);
                    }
                }
                EventData::Author(value) => {
                    if is_root_scope {
                        rootdata.author = Some(*value);
                    }
                }
                EventData::Cmd(value) => {
                    is_root_scope = false;
                    let mut cmd = Cmd::default();
                    if !value.is_empty() {
                        cmd.describe = Some(*value);
                    }
                    maybe_subcmd = Some(cmd);
                }
                EventData::Aliases(values) => {
                    if let Some(cmd) = &mut maybe_subcmd {
                        for name in values {
                            if let Some(pos) = rootdata.names.get(name) {
                                bail!(
                                    "@alias(line {}) is conflicted with cmd or alias at line {}",
                                    position,
                                    pos
                                );
                            }
                            rootdata.names.insert(name, *position);
                        }
                        cmd.aliases = values.to_vec();
                    } else {
                        bail!("@alias(line {}) is unexpected", position)
                    }
                }
                EventData::Arg(arg_data) => {
                    if let Some(cmd) = &mut maybe_subcmd {
                        cmd.add_arg(arg_data, position)?;
                    } else if is_root_scope {
                        rootcmd.add_arg(arg_data, position)?;
                    } else {
                        bail!(
                            "{}(line {}) is unexpected, maybe miss @cmd?",
                            arg_data.kind,
                            position
                        )
                    }
                }
                EventData::Func(name) => {
                    is_root_scope = false;
                    if let Some(mut cmd) = maybe_subcmd.take() {
                        if let Some(pos) = rootdata.names.get(name) {
                            bail!(
                                "{}(line {}) is conflicted with cmd or alias at line {}",
                                name,
                                position,
                                pos
                            )
                        }
                        rootdata.names.insert(name, *position);
                        cmd.name = Some((name, name.to_case(Case::Kebab)));
                        rootcmd.cmds.push(cmd);
                    } else if *name == ENTRYPOINT {
                        rootdata.main = true;
                    }
                }
                EventData::Unknown(name) => {
                    bail!("@{}(line {}) is unknown", name, position);
                }
            }
        }
        rootcmd.root = Some(rootdata);
        Ok(rootcmd)
    }
    fn build(&'a self, name: &'a str) -> Result<Command<'a>> {
        let mut cmd = Command::new(name);
        if let Some(describe) = self.describe {
            cmd = cmd.about(describe);
        }
        if let Some(rootdata) = &self.root {
            if let Some(version) = rootdata.version {
                cmd = cmd.version(version);
            }
            if let Some(author) = rootdata.author {
                cmd = cmd.author(author);
            }
            if !self.cmds.is_empty() && !rootdata.main {
                cmd = cmd.subcommand_required(true).arg_required_else_help(true);
            }
        }
        if !self.aliases.is_empty() {
            cmd = cmd.visible_aliases(&self.aliases);
        }
        for arg_data in &self.args {
            cmd = cmd.arg(arg_data.build()?);
        }
        for subcmd in &self.cmds {
            let subcmd = subcmd.build(subcmd.name.as_ref().unwrap().1.as_str())?;
            cmd = cmd.subcommand(subcmd);
        }
        Ok(cmd)
    }
    fn retrieve(&'a self, matches: &ArgMatches, runner: &Runner) -> String {
        let mut values = vec![];
        for arg_data in &self.args {
            if let Some(value) = arg_data.retrieve_match_value(matches) {
                if !value.is_empty() {
                    values.push(value);
                }
            }
        }
        let mut call_fn: Option<String> = None;
        for subcmd in &self.cmds {
            if let Some((fn_name, cmd_name)) = &subcmd.name {
                if let Some((match_name, subcmd_matches)) = matches.subcommand() {
                    if cmd_name.as_str() == match_name {
                        let value = subcmd.retrieve(subcmd_matches, runner);
                        if !value.is_empty() {
                            values.push(value);
                        }
                        call_fn = Some(fn_name.to_string());
                    }
                }
            }
        }
        call_fn = call_fn.or_else(|| {
            self.root.as_ref().and_then(|v| {
                if v.main {
                    Some("main".to_string())
                } else {
                    None
                }
            })
        });
        if let Some(fn_name) = call_fn {
            if runner.eval {
                values.push(fn_name);
            } else {
                values.push(format!("{}__{}={}", VARIABLE_PREFIX, "call", fn_name));
            }
        }
        values.join("\n")
    }
    fn add_arg(&mut self, arg_data: &'a ArgData, position: &Position) -> Result<()> {
        let arg_data = WrapArgData::new(arg_data, self.positional_idx);
        arg_data.detect_conflict(&mut self.names, *position)?;
        if arg_data.is_positional() {
            self.positional_idx += 1;
        }
        self.args.push(arg_data);
        Ok(())
    }
}

#[derive(Debug)]
struct WrapArgData<'a> {
    data: &'a ArgData<'a>,
    value_name: String,
    pos_index: usize,
}

impl<'a> Deref for WrapArgData<'a> {
    type Target = ArgData<'a>;
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a> WrapArgData<'a> {
    fn new(data: &'a ArgData<'a>, pos_index: usize) -> Self {
        let value_name = data
            .value_name
            .map(|v| v.to_owned())
            .unwrap_or_else(|| data.name.to_case(Case::Cobol));
        Self {
            data,
            value_name,
            pos_index,
        }
    }
    fn build(&'a self) -> Result<Arg<'a>> {
        let mut arg = Arg::new(self.name);
        if let Some(summary) = self.summary {
            let title = summary.trim();
            if !title.is_empty() {
                arg = arg.help(title);
            }
        }
        let arg = match &self.kind {
            ArgKind::Flag => {
                let mut arg = arg.long(self.name);
                if let Some(s) = self.short {
                    arg = arg.short(s);
                }
                arg
            }
            ArgKind::Option => {
                let mut arg = arg
                    .long(self.name)
                    .required(self.required)
                    .value_name(&self.value_name);
                if let Some(s) = self.short {
                    arg = arg.short(s);
                }
                if self.multiple {
                    arg = arg
                        .multiple_values(true)
                        .use_value_delimiter(true)
                        .multiple_occurrences(true);
                }
                if let Some(choices) = &self.choices {
                    if choices.len() > 1 {
                        arg = arg.possible_values(choices);
                    }
                }
                if let Some(default) = self.default {
                    arg = arg.default_value(default);
                }
                arg
            }
            ArgKind::Positional => {
                let mut arg = arg
                    .index(self.pos_index + 1)
                    .required(self.required)
                    .value_name(&self.value_name);
                if let Some(choices) = &self.choices {
                    if choices.len() > 1 {
                        arg = arg.possible_values(choices);
                    }
                }
                if self.multiple {
                    arg = arg.multiple_values(true)
                }
                arg
            }
        };
        Ok(arg)
    }
    fn retrieve_match_value(&self, matches: &ArgMatches) -> Option<String> {
        let name = self.name.to_case(Case::Snake);
        if !matches.is_present(self.name) {
            return None;
        }
        if self.kind == ArgKind::Flag {
            return Some(format!("{}_{}=1", VARIABLE_PREFIX, name));
        }
        if self.multiple {
            return matches.values_of(self.name).map(|values| {
                let values: Vec<String> = values.map(escape_value).collect();
                format!("{}_{}=( {} )", VARIABLE_PREFIX, name, values.join(" "))
            });
        }
        matches
            .value_of(self.name)
            .map(|value| format!("{}_{}={}", VARIABLE_PREFIX, name, escape_value(value)))
    }
    fn detect_conflict(
        &self,
        names: &mut (HashMap<&'a str, Position>, HashMap<char, Position>),
        current: Position,
    ) -> Result<()> {
        match self.kind {
            ArgKind::Positional => {
                if let Some(position) = names.0.get(self.name) {
                    bail!(
                        "{}(line {}) has `{}` already exists at line {}",
                        self.kind,
                        current,
                        self.name,
                        position,
                    );
                } else {
                    names.0.insert(self.name, current);
                }
            }
            _ => {
                if let Some(position) = names.0.get(self.name) {
                    bail!(
                        "{}(line {}) has --{} already exists at line {}",
                        self.kind,
                        current,
                        self.name,
                        position,
                    )
                } else {
                    names.0.insert(self.name, current);
                }
                if let Some(short) = self.short {
                    if let Some(position) = names.1.get(&short) {
                        bail!(
                            "{}(line {}) has -{} already exists at line {}",
                            self.kind,
                            current,
                            short,
                            position,
                        )
                    } else {
                        names.1.insert(short, current);
                    }
                }
            }
        }
        Ok(())
    }
}

fn escape_value(value: &str) -> String {
    let mut output = String::new();
    if value.is_empty() {
        return "''".to_string();
    }
    for ch in value.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' | ',' | ':' | '/' | '@' => {
                output.push(ch)
            }
            '\n' => output.push_str("'\n'"),
            _ => {
                output.push('\\');
                output.push(ch);
            }
        }
    }
    output
}
