use std::{cell::RefCell, collections::HashMap};

use slotmap::{new_key_type, SlotMap};

use crate::process::{LuaValue, TableValue};

new_key_type! {
    pub(crate) struct LuaValueRef;
}

new_key_type! {
    pub struct TableRef;
}

#[derive(Debug, Default)]
pub(crate) struct EvaluatorStorage {
    storage: SlotMap<LuaValueRef, LuaValue>,
    identifiers: Vec<HashMap<String, LuaValueRef>>,
    tables: RefCell<SlotMap<TableRef, TableValue<'static>>>,
}

impl EvaluatorStorage {
    pub(crate) fn push_scope(&mut self) {
        self.identifiers.push(Default::default())
    }

    pub(crate) fn pop_scope(&mut self) {
        self.identifiers.pop();
    }

    pub(crate) fn create_table(&self, table: TableValue<'static>) -> LuaValue {
        let table_ref = self.tables.borrow_mut().insert(table);
        LuaValue::Table(table_ref)
    }

    pub(crate) fn read_table<T>(
        &self,
        table_ref: TableRef,
        function: impl FnOnce(&TableValue<'_>) -> T,
    ) -> Option<T> {
        self.tables.borrow().get(table_ref).map(function)
    }

    pub(crate) fn mark_mutated(&mut self, identifier: &str) {
        if let Some(map) = self
            .identifiers
            .iter_mut()
            .rev()
            .find_map(|map| map.contains_key(identifier).then_some(map))
        {
            let new_value_ref = self.storage.insert(LuaValue::Unknown);
            map.insert(identifier.to_owned(), new_value_ref);
        }
    }

    pub(crate) fn read_identifier(&self, identifier: &str) -> LuaValue {
        if let Some(value_ref) = self
            .identifiers
            .iter()
            .rev()
            .find_map(|map| map.get(identifier))
        {
            self.storage
                .get(*value_ref)
                .map(Clone::clone)
                .unwrap_or(LuaValue::Unknown)
        } else {
            // todo: add a parameter to set undefined global identifiers to nil
            LuaValue::Unknown
        }
    }

    pub(crate) fn declare_identifier(
        &mut self,
        identifier: &str,
        value: Option<LuaValue>,
    ) -> LuaValueRef {
        let value_ref = self.storage.insert(value.unwrap_or_else(|| LuaValue::Nil));
        self.insert_identifier_ref(identifier, value_ref);
        value_ref
    }

    fn insert_identifier_ref(&mut self, identifier: &str, value_ref: LuaValueRef) {
        if let Some(map) = self.identifiers.last_mut() {
            map.insert(identifier.to_string(), value_ref);
        } else {
            let mut set = HashMap::new();
            set.insert(identifier.to_string(), value_ref);
            self.identifiers.push(set);
        }
    }
}
