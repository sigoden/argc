mod fixtures;

pub const SCRIPT_OPTIONS: &str = include_str!("../examples/options.sh");
pub const SCRIPT_ARGS: &str = include_str!("../examples/args.sh");
pub const SCRIPT_ENVS: &str = include_str!("../examples/envs.sh");
pub const SCRIPT_DETAILS: &str = include_str!("../examples/details.sh");

pub use fixtures::locate_script;

#[macro_use]
mod macros;
mod argcfile;
mod cli;
mod compgen;
mod create;
mod details;
mod env;
mod export;
mod fail;
mod hook_fn;
mod main_fn;
mod meta;
mod misc;
mod parallel;
mod param_fn;
mod spec;
mod validate;
