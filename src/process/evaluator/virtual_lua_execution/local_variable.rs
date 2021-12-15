use super::LuaValue;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalVariable {
    mutable: bool,
    current_value: LuaValue,
}

impl LocalVariable {
    pub fn new(value: LuaValue) -> LocalVariable {
        Self {
            mutable: false,
            current_value: value,
        }
    }

    pub fn assign(&mut self, value: LuaValue) {
        self.mutable = true;
        self.current_value = value;
    }

    pub fn get_value(&self) -> LuaValue {
        self.current_value.clone()
    }
}
