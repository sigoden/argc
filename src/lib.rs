mod cli;
mod arg;
mod token;

use anyhow::bail as throw;
use anyhow::Error;

pub use cli::Cli;

pub type Result<T> = std::result::Result<T, Error>;
