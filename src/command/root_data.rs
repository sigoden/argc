use crate::parser::{EventScope, Position};

use anyhow::{bail, Result};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RootData {
    pub(crate) scope: EventScope,
    pub(crate) fns: HashMap<String, Position>,
    pub(crate) cmd_fns: HashMap<String, Position>,
    pub(crate) cmd_pos: usize,
    pub(crate) default_fns: Vec<(String, Position)>,
    pub(crate) choices_fns: Vec<(String, Position)>,
}

impl RootData {
    pub(crate) fn add_param_fn(
        &mut self,
        position: usize,
        default_fn: &Option<String>,
        choices_fn: &Option<(String, bool)>,
    ) {
        if let Some(default_fn) = default_fn.as_ref() {
            self.default_fns.push((default_fn.to_string(), position));
        }
        if let Some((choices_fn, _)) = choices_fn.as_ref() {
            self.choices_fns.push((choices_fn.to_string(), position));
        }
    }

    pub(crate) fn check_param_fn(&self) -> Result<()> {
        for (name, pos) in self.default_fns.iter() {
            if !self.fns.contains_key(name) {
                bail!("{}(line {}) is missing", name, pos,)
            }
        }
        for (name, pos) in self.choices_fns.iter() {
            if !self.fns.contains_key(name) {
                bail!("{}(line {}) is missing", name, pos,)
            }
        }
        Ok(())
    }
}
