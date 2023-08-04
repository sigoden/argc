use crate::parser::{EventScope, Position};

use anyhow::{bail, Result};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub(crate) struct RootData {
    pub(crate) scope: EventScope,
    pub(crate) fns: HashMap<String, Position>,
    pub(crate) cmd_fns: HashMap<String, Position>,
    pub(crate) cmd_pos: usize,
    pub(crate) default_fns: Vec<(String, Position)>,
    pub(crate) choice_fns: Vec<(String, Position)>,
}

impl RootData {
    pub(crate) fn add_param_fn(
        &mut self,
        position: usize,
        default_fn: Option<&String>,
        choice_fn: Option<(&String, &bool)>,
    ) {
        if let Some(f) = default_fn {
            self.default_fns.push((f.to_string(), position));
        }
        if let Some((f, _)) = choice_fn {
            self.choice_fns.push((f.to_string(), position));
        }
    }

    pub(crate) fn check_param_fn(&self) -> Result<()> {
        for (name, pos) in self.default_fns.iter() {
            if !self.fns.contains_key(name) {
                bail!("{}(line {}) is missing", name, pos,)
            }
        }
        for (name, pos) in self.choice_fns.iter() {
            if !self.fns.contains_key(name) {
                bail!("{}(line {}) is missing", name, pos,)
            }
        }
        Ok(())
    }
}
