use std::{collections::HashSet, mem};

use crate::process::{FunctionValue, LuaValue};

use super::TableId;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionEffect {
    mutated_identifiers: Vec<Vec<String>>,
}

impl ExecutionEffect {
    pub fn add<S: Into<String>>(&mut self, identifier: S) {
        if let Some(identifiers) = self.mutated_identifiers.last_mut() {
            identifiers.push(identifier.into());
        }
    }

    pub fn enable(&mut self) {
        self.mutated_identifiers.push(Vec::new());
    }

    pub fn disable(&mut self) -> impl Iterator<Item = String> {
        if let Some(identifiers) = self.mutated_identifiers.pop() {
            identifiers.into_iter()
        } else {
            Vec::new().into_iter()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionSideEffect {
    state_parent: Vec<usize>,
    has_side_effects: Vec<bool>,
}

impl ExecutionSideEffect {
    pub fn add(&mut self) {
        if let Some(value) = self.has_side_effects.last_mut() {
            if !*value {
                *value = true;
            }
        }
    }

    pub fn current_state(&self) -> Option<usize> {
        self.state_parent.last().map(ToOwned::to_owned)
    }

    pub fn enable(&mut self, parent_id: usize) {
        self.has_side_effects.push(false);
        self.state_parent.push(parent_id);
    }

    pub fn disable(&mut self) -> bool {
        if let Some(value) = self.has_side_effects.pop() {
            self.state_parent.pop();
            value
        } else {
            false
        }
    }

    pub fn enable_within_state(&mut self) {
        self.has_side_effects.push(false);
    }

    pub fn disable_within_state(&mut self) -> bool {
        self.has_side_effects.pop().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArgumentEffect {
    visited: HashSet<TableId>,
    functions: Vec<FunctionValue>,
    table_ids: Vec<TableId>,
}

impl ArgumentEffect {
    pub fn insert(&mut self, value: LuaValue) {
        match value {
            LuaValue::Function(function) => {
                self.functions.push(function);
            }
            LuaValue::TableRef(id) => {
                if !self.visited.contains(&id) {
                    self.visited.insert(id);
                    self.table_ids.push(id);
                }
            }
            LuaValue::False
            | LuaValue::Table(_)
            | LuaValue::Nil
            | LuaValue::Number(_)
            | LuaValue::String(_)
            | LuaValue::True
            | LuaValue::Tuple(_)
            | LuaValue::Unknown => {}
        }
    }

    pub fn drain(&mut self) -> (Vec<TableId>, Vec<FunctionValue>) {
        let table_ids = mem::take(&mut self.table_ids);
        let functions = mem::take(&mut self.functions);
        (table_ids, functions)
    }

    pub fn is_empty(&self) -> bool {
        self.functions.is_empty() && self.table_ids.is_empty()
    }
}
