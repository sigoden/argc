mod argc_value;
mod command;
mod compgen;
mod matcher;
mod param;
mod parser;
pub mod utils;

use anyhow::Error;
pub use argc_value::{ArgcValue, VARIABLE_PREFIX};
pub use command::{eval, export};
pub use compgen::{compgen, Shell};

pub type Result<T> = std::result::Result<T, Error>;
