mod arg;
mod cli;
mod parser;

use anyhow::bail;
use anyhow::Error;
use arg::ArgData;

pub use cli::Cli;

pub type Result<T> = std::result::Result<T, Error>;
