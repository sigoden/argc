mod argc_value;
#[cfg(feature = "build")]
mod build;
mod command;
#[cfg(feature = "compgen")]
mod compgen;
#[cfg(feature = "completions")]
mod completions;
#[cfg(feature = "mangen")]
mod mangen;
#[cfg(any(feature = "eval", feature = "compgen"))]
mod matcher;
mod param;
mod parser;
mod runtime;
#[cfg(any(feature = "compgen", feature = "completions"))]
mod shell;
pub mod utils;

use anyhow::Result;
pub use argc_value::ArgcValue;
#[cfg(feature = "build")]
pub use build::build;
#[cfg(feature = "export")]
pub use command::CommandValue;
#[cfg(feature = "compgen")]
pub use compgen::compgen;
#[cfg(feature = "completions")]
pub use completions::generate_completions;
#[cfg(feature = "mangen")]
pub use mangen::mangen;
pub use param::{ChoiceValue, DefaultValue};
#[cfg(feature = "export")]
pub use param::{EnvValue, FlagOptionValue, PositionalValue};
#[cfg(feature = "native-runtime")]
pub use runtime::navite::NativeRuntime;
pub use runtime::Runtime;
#[cfg(any(feature = "compgen", feature = "completions"))]
pub use shell::Shell;

#[cfg(feature = "eval")]
pub fn eval<T: Runtime>(
    runtime: T,
    script_content: &str,
    args: &[String],
    script_path: Option<&str>,
    wrap_width: Option<usize>,
) -> Result<Vec<ArgcValue>> {
    let mut cmd = command::Command::new(script_content, &args[0])?;
    cmd.eval(runtime, args, script_path, wrap_width)
}

#[cfg(feature = "export")]
pub fn export(source: &str, root_name: &str) -> Result<CommandValue> {
    let cmd = command::Command::new(source, root_name)?;
    Ok(cmd.export())
}
