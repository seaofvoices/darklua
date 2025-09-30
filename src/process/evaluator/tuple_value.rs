use crate::process::LuaValue;

pub struct TupleValue {
    values: Vec<LuaValue>,
    extend_unknown: bool,
}

impl TupleValue {
    pub fn known_values(values: Vec<LuaValue>) -> Self {
        let extend_unknown = values
            .iter()
            .last()
            .is_some_and(|value| *value == LuaValue::Unknown);
        Self {
            values,
            extend_unknown,
        }
    }

    pub fn unknown() -> Self {
        Self {
            values: Default::default(),
            extend_unknown: true,
        }
    }

    pub fn empty() -> Self {
        Self {
            values: Default::default(),
            extend_unknown: false,
        }
    }

    pub(crate) fn prepend(&mut self, mut values: Vec<LuaValue>) {
        std::mem::swap(&mut self.values, &mut values);
        self.values.extend(values);
    }

    pub fn insert(&mut self, index: usize, value: LuaValue) {
        if index >= self.values.len() {
            self.values.push(value);
        } else {
            self.values.insert(index, value);
        }
    }

    pub fn into_values(self) -> impl Iterator<Item = LuaValue> {
        self.values
            .into_iter()
            .chain(std::iter::repeat(if self.extend_unknown {
                LuaValue::Unknown
            } else {
                LuaValue::Nil
            }))
    }

    pub fn into_one(self) -> LuaValue {
        self.values
            .into_iter()
            .next()
            .unwrap_or(if self.extend_unknown {
                LuaValue::Unknown
            } else {
                LuaValue::Nil
            })
    }
}

impl From<LuaValue> for TupleValue {
    fn from(value: LuaValue) -> Self {
        if let LuaValue::Unknown = value {
            Self::unknown()
        } else {
            Self::known_values(vec![value])
        }
    }
}
