use clap::{Arg, ArgMatches};
use convert_case::{Case, Casing};

#[derive(Debug, PartialEq, Clone)]
pub struct ArgData<'a> {
    pub name: &'a str,
    pub summary: Option<&'a str>,
    pub flag: bool,
    pub short: Option<char>,
    pub choices: Option<Vec<&'a str>>,
    pub multiple: bool,
    pub required: bool,
    pub positional: bool,
    pub default: Option<&'a str>,
}

impl<'a> ArgData<'a> {
    pub fn new(name: &'a str) -> Self {
        ArgData {
            name,
            summary: None,
            flag: false,
            short: None,
            choices: None,
            multiple: false,
            required: false,
            positional: false,
            default: None,
        }
    }
    pub fn build(&'a self, index: usize) -> Arg<'a> {
        let mut arg = Arg::new(self.name)
            .takes_value(!self.flag)
            .required(self.required)
            .multiple_values(self.multiple);
        if let Some(summary) = self.summary {
            let title = summary.trim();
            if title.len() > 0 {
                arg = arg.help(title);
            }
        }
        if self.positional {
            arg = arg.index(index + 1);
        } else {
            arg = arg.long(self.name);
            if let Some(short) = self.short {
                arg = arg.short(short);
            }
            if let Some(choices) = &self.choices {
                if choices.len() > 1 {
                    arg = arg.possible_values(choices);
                }
            }
            if let Some(default) = self.default {
                arg = arg.default_value(default);
            }
        }
        arg
    }
    pub fn retrive(&'a self, matches: &ArgMatches) -> Option<String> {
        let name = self.name.to_case(Case::Snake);
        if !matches.is_present(self.name) {
            return None;
        }
        if self.flag {
            return Some(format!("argc_{}=1\n", name));
        }
        if self.multiple {
            return matches.values_of(self.name).map(|values| {
                let values: Vec<&str> = values.collect();
                format!("argc_{}=( {} )\n", name, values.join(" "))
            });
        }
        matches
            .value_of(self.name)
            .map(|value| format!("argc_{}={}\n", name, value))
    }
}
