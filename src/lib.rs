mod argc_value;
mod build;
mod command;
mod compgen;
mod matcher;
mod param;
mod parser;
pub mod utils;

use anyhow::Result;
pub use argc_value::ArgcValue;
pub use build::build;
pub use command::CommandValue;
pub use compgen::{compgen, Shell};
pub use param::{ChoiceValue, DefaultValue, EnvValue, FlagOptionValue, PositionalValue};

pub fn eval(
    script_content: &str,
    args: &[String],
    script_path: Option<&str>,
    term_width: Option<usize>,
) -> Result<Vec<ArgcValue>> {
    let mut cmd = command::Command::new(script_content, &args[0])?;
    cmd.eval(args, script_path, term_width)
}

pub fn export(source: &str, root_name: &str) -> Result<CommandValue> {
    let cmd = command::Command::new(source, root_name)?;
    Ok(cmd.export())
}
