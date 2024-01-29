mod argc_value;
mod command;
mod compgen;
mod matcher;
mod param;
mod parser;
pub mod utils;

use anyhow::Result;
pub use argc_value::{ArgcValue, VARIABLE_PREFIX};
pub use compgen::{compgen, Shell};

pub fn eval(
    script_content: &str,
    args: &[String],
    script_path: Option<&str>,
    term_width: Option<usize>,
) -> Result<Vec<ArgcValue>> {
    let mut cmd = command::Command::new(script_content)?;
    cmd.eval(args, script_path, term_width)
}

pub fn export(source: &str) -> Result<serde_json::Value> {
    let cmd = command::Command::new(source)?;
    Ok(cmd.to_json())
}
