/// Represents an evaluated Expression result.
#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    False,
    Function,
    Nil,
    Number(f64),
    String(String),
    Table,
    True,
    Unknown,
}

impl LuaValue {
    /// As defined in Lua, all values are considered true, except for false and nil. An option is
    /// returned as the LuaValue may be unknown, so it would return none.
    /// ```rust
    /// # use darklua_core::process::LuaValue;
    ///
    /// // the values considered false
    /// assert!(!LuaValue::False.is_truthy().unwrap());
    /// assert!(!LuaValue::Nil.is_truthy().unwrap());
    ///
    /// // all the others are true
    /// assert!(LuaValue::True.is_truthy().unwrap());
    /// assert!(LuaValue::Table.is_truthy().unwrap());
    /// assert!(LuaValue::Number(0.0).is_truthy().unwrap());
    /// assert!(LuaValue::String("hello".to_owned()).is_truthy().unwrap());
    ///
    /// // unknown case
    /// assert!(LuaValue::Unknown.is_truthy().is_none());
    /// ```
    pub fn is_truthy(&self) -> Option<bool> {
        match self {
            Self::Unknown => None,
            Self::Nil | Self::False => Some(false),
            _ => Some(true),
        }
    }

    /// If the value is unknown, this will also return unknown value. In the other case, if the
    /// value is considered truthy (see `is_truthy` function), it will call the given function to
    /// get the mapped value.
    pub fn map_if_truthy<F>(self, map: F) -> Self where F: Fn(Self) -> Self {
        match self.is_truthy() {
            Some(true) => map(self),
            Some(false) => self,
            _ => Self::Unknown,
        }
    }

    /// Like the `map_if_truthy` method, except that instead of returning the same value when the
    /// it is falsy, it calls the default callback to obtain another value.
    pub fn map_if_truthy_else<F, G>(self, map: F, default: G) -> Self
        where F: Fn(Self) -> Self, G: Fn() -> Self
    {
        match self.is_truthy() {
            Some(true) => map(self),
            Some(false) => default(),
            _ => Self::Unknown,
        }
    }
}

impl Default for LuaValue {
    fn default() -> Self { Self::Unknown }
}

impl From<bool> for LuaValue {
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }
}

impl From<String> for LuaValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for LuaValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<f64> for LuaValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unknown_lua_value_is_truthy_returns_none() {
        assert!(LuaValue::Unknown.is_truthy().is_none());
    }

    #[test]
    fn false_value_is_not_truthy() {
        assert!(!LuaValue::False.is_truthy().unwrap());
    }

    #[test]
    fn nil_value_is_not_truthy() {
        assert!(!LuaValue::Nil.is_truthy().unwrap());
    }

    #[test]
    fn true_value_is_truthy() {
        assert!(LuaValue::True.is_truthy().unwrap());
    }

    #[test]
    fn zero_value_is_truthy() {
        assert!(LuaValue::Number(0_f64).is_truthy().unwrap());
    }

    #[test]
    fn string_value_is_truthy() {
        assert!(LuaValue::String("".to_owned()).is_truthy().unwrap());
    }
}
