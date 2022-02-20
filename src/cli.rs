use crate::parser::{parse, Arg};
use crate::Result;

#[derive(Debug)]
pub struct Cli<'a> {
    pub title: Option<&'a str>,
    pub options: Vec<Arg<'a>>,
    pub params: Vec<Arg<'a>>,
}

#[derive(Debug)]
pub struct SubCommand<'a> {
    pub name: String,
    pub title: Option<String>,
    pub options: Vec<Arg<'a>>,
    pub params: Vec<Arg<'a>>,
}

impl<'a> Cli<'a> {
    /// Create app from bash script
    pub fn new(source: &str) -> Result<Self> {
        let tokens = parse(source)?;
        todo!()
    }
}
