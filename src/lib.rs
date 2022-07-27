mod cli;
mod param;
mod parser;
mod utils;

use anyhow::Error;
pub use cli::Cli;

pub type Result<T> = std::result::Result<T, Error>;
