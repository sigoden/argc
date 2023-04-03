mod fixtures;

const SPEC_SCRIPT: &str = include_str!("spec.sh");

#[macro_use]
mod macros;
mod argcfile;
mod cli;
mod compgen;
mod create;
mod escape_test;
mod export;
mod fail_test;
#[cfg(unix)]
mod interrupt;
mod main_fn_test;
mod param_fn_test;
mod spec_test;
