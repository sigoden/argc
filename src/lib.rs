mod cli;
mod parser;
mod utils;

use anyhow::Error;
pub use cli::{run, Runner};

pub type Result<T> = std::result::Result<T, Error>;
