mod cli;
mod parser;

use anyhow::Error;
pub use cli::run;

pub type Result<T> = std::result::Result<T, Error>;
