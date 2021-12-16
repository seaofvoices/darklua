use std::mem;

use super::LuaValue;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableValue {
    array: Vec<LuaValue>,
    pairs: Vec<(LuaValue, LuaValue)>,
    unknown_mutations: bool,
    // metatable: Option<TableValue>
}

impl TableValue {
    pub fn with_array_element(mut self, value: LuaValue) -> Self {
        self.array.push(value);
        self
    }

    pub fn with_entry<T: Into<LuaValue>, U: Into<LuaValue>>(mut self, key: T, value: U) -> Self {
        self.insert_entry(key.into(), value.into());
        self
    }

    pub fn set_unknown_mutations(&mut self) {
        self.unknown_mutations = true;
    }

    pub fn clear(&mut self) {
        self.array.clear();
        self.pairs.clear();
    }

    #[inline]
    pub fn push_element(&mut self, value: LuaValue) {
        self.array.push(value);
    }

    pub fn insert<T: Into<LuaValue>, U: Into<LuaValue>>(&mut self, new_key: T, new_value: U) {
        let new_key = new_key.into();

        if let Some(index) = self.get_array_index(&new_key) {
            if index < self.array.len() {
                self.array[index] = new_value.into();
                return;
            } else if index == self.array.len() {
                self.array.push(new_value.into());
                return;
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

    pub fn get(&self, key: &LuaValue) -> Option<&LuaValue> {
        if let Some(index) = self.get_array_index(key) {
            if index < self.array.len() {
                if let Some(element) = self.array.get(index) {
                    return Some(element);
                }
            }
        }
        self.pairs
            .iter()
            .find(|(existing_key, _)| existing_key == key)
            .map(|(_, value)| value)
            .or_else(|| {
                if self.unknown_mutations {
                    Some(&LuaValue::Unknown)
                } else {
                    None
                }
            })
    }

    fn remove_key(&mut self, key: &LuaValue) {
        self.pairs.retain(|(existing_key, _)| existing_key != key);
    }

    fn get_array_index(&self, key: &LuaValue) -> Option<usize> {
        if let LuaValue::Number(index) = *key {
            if index >= 1.0 && index.trunc() == index {
                Some(index as usize)
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

    #[test]
    fn unknown_lua_value_is_truthy_returns_none() {
        assert!(LuaValue::Unknown.is_truthy().is_none());
    }
}
