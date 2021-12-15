use std::collections::HashMap;

use crate::process::LuaValue;

use super::{local_variable::LocalVariable, VirtualLuaExecution};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    id: usize,
    parent: Option<usize>,
    locals: HashMap<String, LocalVariable>,
}

impl State {
    pub fn new_root(id: usize) -> Self {
        Self {
            id,
            parent: None,
            locals: HashMap::default(),
        }
    }

    pub fn new(id: usize, parent: usize) -> Self {
        Self {
            id,
            parent: Some(parent),
            locals: HashMap::default(),
        }
    }

    pub fn insert_local<S: Into<String>>(&mut self, name: S, value: LuaValue) {
        self.locals.insert(name.into(), LocalVariable::new(value));
    }

    pub fn assign_identifier(&mut self, name: &str, value: LuaValue) {
        if let Some(variable) = self.locals.get_mut(name) {
            variable.assign(value);
        }
    }

    pub fn read(&self, identifier: &str, root_state: &VirtualLuaExecution) -> Option<LuaValue> {
        self.locals
            .get(identifier)
            .map(LocalVariable::get_value)
            .or_else(|| {
                self.parent
                    .and_then(|parent_id| root_state.get_state(parent_id))
                    .and_then(|state| state.read(identifier, root_state))
            })
    }

    pub fn has_identifier(&self, identifier: &str) -> bool {
        self.locals.get(identifier).is_some()
    }

    #[inline]
    pub fn parent(&self) -> Option<usize> {
        self.parent.to_owned()
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }
}
