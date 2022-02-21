mod cli;
mod parser;

use anyhow::bail;
use anyhow::Error;

pub use cli::build;

pub type Result<T> = std::result::Result<T, Error>;
