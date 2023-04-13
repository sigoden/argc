mod argc_value;
mod cli;
mod compgen;
mod completion;
pub mod param;
pub mod parser;
pub mod utils;

use anyhow::Error;
pub use argc_value::ArgcValue;
pub use cli::{eval, export};
pub use compgen::{compgen, Shell};

pub type Result<T> = std::result::Result<T, Error>;
