mod fixtures;

pub const SCRIPT_OPTIONS: &str = include_str!("../examples/options");
pub const SCRIPT_ARGS: &str = include_str!("../examples/args");
pub const SCRIPT_ENVS: &str = include_str!("../examples/envs");
pub const SCRIPT_MULTILINE: &str = include_str!("../examples/multiline");

pub use fixtures::locate_script;

#[macro_use]
mod macros;
mod bind_env;
mod cli;
mod compgen;
mod env;
mod fail;
mod hook_fn;
mod main_fn;
mod meta;
#[cfg(target_os = "linux")]
mod misc;
mod multiline;
mod param_fn;
mod spec;
mod validate;

mod external_subcommands;
