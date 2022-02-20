use crate::arg::Arg;
use crate::Result;
use crate::token::tokenize;

#[derive(Debug)]
pub struct Cli {
    pub title: Option<String>,
    pub version: Option<String>,
    pub options: Vec<Arg>,
    pub commands: Vec<Arg>,
}

impl Cli {
    /// Create app from bash script
    pub fn new(source: &str) -> Result<Self> {
        let tokens = tokenize(source)?;
        todo!()
    }
}
