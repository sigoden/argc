pub mod cmd;
pub mod param;
pub mod parser;
pub mod utils;
pub mod value;

use anyhow::Error;
pub use cmd::{compgen, run};

pub type Result<T> = std::result::Result<T, Error>;
