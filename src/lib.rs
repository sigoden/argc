pub mod argc_value;
pub mod cli;
pub mod completion;
pub mod param;
pub mod parser;
pub mod utils;

use anyhow::Error;
pub use cli::{eval, export};
pub use completion::compgen;

pub type Result<T> = std::result::Result<T, Error>;
