use std::{
    borrow::Cow,
    collections::HashMap,
    hash::{Hash, Hasher},
};

use crate::process::LuaValue;

#[derive(Debug, Clone, Default, PartialEq)]
struct TableKey<'a> {
    value: Cow<'a, LuaValue>,
}

impl<'a> TableKey<'a> {
    fn new(value: Cow<'a, LuaValue>) -> Option<Self> {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();

        try_hash_lua_value(&value, &mut hasher).then(|| Self { value })
    }
}

impl<'a> Eq for TableKey<'a> {}

impl<'a> Hash for TableKey<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if !try_hash_lua_value(&self.value, state) {
            log::warn!("unexpected value used as hash table key");
        }
    }
}

fn try_hash_lua_value<H: Hasher>(value: &LuaValue, state: &mut H) -> bool {
    match value {
        LuaValue::False => false.hash(state),
        LuaValue::True => true.hash(state),
        LuaValue::Number(number) => {
            if number.is_nan() {
                return false;
            }
            number.to_bits().hash(state);
        }
        LuaValue::String(items) => items.hash(state),
        LuaValue::Function(_) | LuaValue::Table(_) | LuaValue::Unknown | LuaValue::Nil => {
            return false
        }
    }
    true
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TableValue<'a> {
    entries: HashMap<TableKey<'a>, LuaValue>,
    // if a table has unknown entries, it will always return unknown values
    // when accessed
    has_unknown_entries: bool,
    // if strict, the table throws an error when unknown keys are accessed
    strict: bool,
    pure_metamethods: bool,
}

impl<'a> TableValue<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_unknown_entries(mut self) -> Self {
        self.has_unknown_entries = true;
        self
    }

    pub fn with_pure_metamethods(mut self) -> Self {
        self.pure_metamethods = true;
        self
    }

    pub fn set_pure_metamethods(&mut self, pure_metamethods: bool) {
        self.pure_metamethods = pure_metamethods;
    }

    pub fn has_pure_metamethods(&self) -> bool {
        self.pure_metamethods
    }

    pub fn get(&self, key: &LuaValue) -> LuaValue {
        if self.has_unknown_entries {
            return LuaValue::Unknown;
        }

        if let Some(hash_key) = TableKey::new(Cow::Borrowed(key)) {
            self.entries
                .get(&hash_key)
                .map(Clone::clone)
                .unwrap_or(if self.strict {
                    LuaValue::Unknown
                } else {
                    LuaValue::Nil
                })
        } else {
            LuaValue::Unknown
        }
    }

    pub fn insert(&mut self, key: LuaValue, value: LuaValue) {
        if self.has_unknown_entries {
            return;
        }

        if let Some(hash_key) = TableKey::new(Cow::Owned(key)) {
            self.entries.insert(hash_key, value);
        } else {
            self.has_unknown_entries = true;
        }
    }
}
