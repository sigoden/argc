mod parser;
mod cli;

use anyhow::Error;
pub use cli::run;

pub type Result<T> = std::result::Result<T, Error>;
