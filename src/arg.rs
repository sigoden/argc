use crate::Result;
use clap::Arg;

#[derive(Debug, PartialEq, Clone)]
pub struct ArgData<'a> {
    pub name: &'a str,
    pub summary: Option<&'a str>,
    pub flag: bool,
    pub short: Option<char>,
    pub choices: Option<Vec<&'a str>>,
    pub multiple: bool,
    pub required: bool,
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
            default: None,
        }
    }
    pub fn build(self, index: Option<usize>) -> Result<Arg<'a>> {
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
        if let Some(idx) = index {
            arg = arg.index(idx);
        } else {
            arg = arg.long(self.name);
        }
        if let Some(short) = self.short {
            arg = arg.short(short);
        }
        if let Some(choices) = self.choices {
            if choices.len() > 1 {
                arg = arg.possible_values(choices);
            }
        }
        if let Some(default) = self.default {
            arg = arg.default_value(default);
        }
        Ok(arg)
    }
}
