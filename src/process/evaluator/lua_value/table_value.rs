use std::{cmp::Ordering, mem};

use crate::process::TableId;

use super::LuaValue;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableValue {
    array: Vec<LuaValue>,
    pairs: Vec<(LuaValue, LuaValue)>,
    unknown_mutations: bool,
    metatable: Option<TableId>,
}

impl TableValue {
    pub fn with_array_element(mut self, value: LuaValue) -> Self {
        self.push_element(value);
        self
    }

    pub fn with_entry<T: Into<LuaValue>, U: Into<LuaValue>>(mut self, key: T, value: U) -> Self {
        self.insert_entry(key.into(), value.into());
        self
    }

    pub fn set_unknown_mutations(&mut self) {
        self.unknown_mutations = true;
    }

    /// Removes all the elements in the array part and the dictionary part of the table.
    pub fn clear(&mut self) {
        self.array.clear();
        self.pairs.clear();
    }

    #[inline]
    pub fn drain_array(&mut self) -> Vec<LuaValue> {
        self.array.drain(..).collect()
    }

    #[inline]
    pub fn drain_entries(&mut self) -> Vec<(LuaValue, LuaValue)> {
        self.pairs.drain(..).collect()
    }

    #[inline]
    /// Adds an element into the array part of the table.
    pub fn push_element(&mut self, value: LuaValue) {
        match value {
            LuaValue::Nil => {}
            LuaValue::Tuple(tuple) => self.push_element(tuple.coerce_to_single_value()),
            LuaValue::Unknown => {
                self.clear();
                self.set_unknown_mutations();
            }
            _ => {
                self.array.push(value);
            }
        }
    }

    /// Inserts into the array part if the key is equal to the next index of the array part, or
    /// adds an entry to the dictionary part.
    pub fn insert<T: Into<LuaValue>, U: Into<LuaValue>>(&mut self, new_key: T, new_value: U) {
        let new_key = new_key.into();
        if matches!(new_key, LuaValue::Unknown) {
            self.clear();
            self.unknown_mutations = true;
            return;
        }

        if let Some(index) = self.get_array_index(&new_key) {
            match index.cmp(&self.array.len()) {
                Ordering::Less => {
                    self.array[index] = new_value.into();
                    return;
                }
                Ordering::Equal => {
                    self.array.push(new_value.into());
                    return;
                }
                Ordering::Greater => {}
            }
        }

        self.insert_entry(new_key, new_value);
    }

    fn insert_entry<T: Into<LuaValue>, U: Into<LuaValue>>(&mut self, new_key: T, new_value: U) {
        let new_key = new_key.into();
        let mut new_value = new_value.into();
        if new_value == LuaValue::Nil {
            self.remove_key(&new_key);
        } else if let Some((_, value)) = self.pairs.iter_mut().find(|(key, _)| key == &new_key) {
            mem::swap(value, &mut new_value);
        } else {
            self.pairs.push((new_key, new_value));
        }
    }

    pub fn get(&self, key: &LuaValue) -> &LuaValue {
        if matches!(key, LuaValue::Unknown) {
            return &LuaValue::Unknown;
        }
        if let Some(index) = self.get_array_index(key) {
            if index < self.array.len() {
                if let Some(element) = self.array.get(index) {
                    return element;
                }
            }
        }
        self.pairs
            .iter()
            .find(|(existing_key, _)| existing_key == key)
            .map(|(_, value)| value)
            .unwrap_or_else(|| {
                if self.unknown_mutations {
                    &LuaValue::Unknown
                } else {
                    &LuaValue::Nil
                }
            })
    }

    fn remove_key(&mut self, key: &LuaValue) {
        self.pairs.retain(|(existing_key, _)| existing_key != key);
    }

    fn get_array_index(&self, key: &LuaValue) -> Option<usize> {
        if let LuaValue::Number(index) = *key {
            if index >= 1.0 && index.trunc() == index {
                Some(index as usize - 1)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn clear_removes_all_elements() {
        let mut table = TableValue::default();
        table.push_element(LuaValue::True);
        table.clear();
        assert_eq!(table, TableValue::default());
    }

    #[test]
    fn push_nil_does_not_add_an_element() {
        let mut table = TableValue::default();
        table.push_element(LuaValue::Nil);
        assert_eq!(table, TableValue::default());
    }

    #[test]
    fn get_first_item_in_array() {
        let mut table = TableValue::default();
        table.push_element(LuaValue::True);
        assert_eq!(table.get(&LuaValue::from(1.0)), &LuaValue::True);
    }

    #[test]
    fn get_known_key_without_value_returns_nil() {
        let mut table = TableValue::default();
        table.push_element(LuaValue::True);
        assert_eq!(table.get(&LuaValue::False), &LuaValue::Nil);
    }

    #[test]
    fn get_unknown_key_returns_unknown() {
        let mut table = TableValue::default();
        table.push_element(LuaValue::True);
        assert_eq!(table.get(&LuaValue::Unknown), &LuaValue::Unknown);
    }
}
