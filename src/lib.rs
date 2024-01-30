mod argc_value;
mod command;
mod compgen;
mod matcher;
mod param;
mod parser;
pub mod utils;

use anyhow::Result;
pub use argc_value::ArgcValue;
pub use command::{CommandValue, GlobalValue};
pub use compgen::{compgen, Shell};
pub use param::{ChoiceValue, DefaultValue, EnvValue, FlagOptionValue, PositionalValue};

pub fn eval(
    script_content: &str,
    args: &[String],
    script_path: Option<&str>,
    term_width: Option<usize>,
) -> Result<Vec<ArgcValue>> {
    let mut cmd = command::Command::new(script_content)?;
    cmd.eval(args, script_path, term_width)
}

pub fn export(source: &str) -> Result<CommandValue> {
    let cmd = command::Command::new(source)?;
    Ok(cmd.export())
}
