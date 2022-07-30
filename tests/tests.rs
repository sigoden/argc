const SPEC_SCRIPT: &str = include_str!("spec.sh");

mod fixtures;

#[macro_use]
mod macros;
mod argcfile;
mod cli;
mod compgen;
mod escape_test;
mod fail_test;
mod help_tag_test;
mod main_fn_test;
mod spec_test;
