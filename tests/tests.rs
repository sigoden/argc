mod fixtures;

pub const SCRIPT_OPTIONS: &str = include_str!("scripts/options.sh");
pub const SCRIPT_ARGS: &str = include_str!("scripts/args.sh");

pub use fixtures::locate_script;

#[macro_use]
mod macros;
mod argcfile;
mod cli;
mod compgen;
mod create;
mod export;
mod fail;
mod main_fn;
mod misc;
mod parallel;
mod param_fn;
mod spec;
mod validate;
mod wrap_help;

#[cfg(unix)]
mod interrupt;
