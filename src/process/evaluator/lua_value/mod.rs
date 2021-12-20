pub mod engine_impl;
mod function_value;
mod table_value;
mod tuple_value;

use std::iter::FromIterator;

use crate::nodes::{Expression, NumberExpression, StringExpression};

pub use function_value::*;
pub use table_value::*;
pub use tuple_value::*;

/// Represents an evaluated Expression result.
#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    False,
    Function,
    Function2(FunctionValue),
    Nil,
    Number(f64),
    String(String),
    Table(TableValue),
    TableRef(usize),
    True,
    Tuple(TupleValue),
    Unknown,
}

impl Eq for LuaValue {}

impl LuaValue {
    pub fn empty_table() -> Self {
        Self::Table(TableValue::default())
    }

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
    pub fn map_if_truthy<F>(self, map: F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        match self.is_truthy() {
            Some(true) => map(self),
            Some(false) => self,
            _ => Self::Unknown,
        }
    }

    /// Like the `map_if_truthy` method, except that instead of returning the same value when the
    /// it is falsy, it calls the default callback to obtain another value.
    pub fn map_if_truthy_else<F, G>(self, map: F, default: G) -> Self
    where
        F: Fn(Self) -> Self,
        G: Fn() -> Self,
    {
        match self.is_truthy() {
            Some(true) => map(self),
            Some(false) => default(),
            _ => Self::Unknown,
        }
    }

    /// Attempt to convert the Lua value into an expression node.
    pub fn to_expression(&self) -> Option<Expression> {
        match self {
            Self::False => Some(Expression::from(false)),
            Self::True => Some(Expression::from(true)),
            Self::Nil => Some(Expression::nil()),
            Self::String(value) => Some(StringExpression::from_value(value).into()),
            Self::Number(value) => Some(Expression::from(*value)),
            Self::Tuple(tuple) => {
                if tuple.len() == 1 {
                    tuple.iter().next().and_then(|value| value.to_expression())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Attempt to convert the Lua value into a number value. This will convert strings when
    /// possible and return the same value otherwise.
    pub fn number_coercion(self) -> Self {
        let single_value = self.coerce_to_single_value();
        match &single_value {
            Self::String(string) => {
                let string = string.trim();

                let number = if string.starts_with('-') {
                    string
                        .get(1..)
                        .and_then(|string| string.parse::<NumberExpression>().ok())
                        .map(|number| number.compute_value() * -1.0)
                } else {
                    string
                        .parse::<NumberExpression>()
                        .ok()
                        .map(|number| number.compute_value())
                };

                number.map(LuaValue::Number)
            }
            _ => None,
        }
        .unwrap_or(single_value)
    }

    /// Attempt to convert the Lua value into a string value. This will convert numbers when
    /// possible and return the same value otherwise.
    pub fn string_coercion(self) -> Self {
        let single_value = self.coerce_to_single_value();
        match &single_value {
            Self::Number(value) => Some(Self::String(format!("{}", value))),
            _ => None,
        }
        .unwrap_or(single_value)
    }

    /// This method only has effect on the Tuple variant. If the value is a tuple, then it will
    /// return only the first value from the tuple, or `nil` if the tuple is empty.
    pub fn coerce_to_single_value(self) -> Self {
        match self {
            Self::Tuple(tuple) => tuple.coerce_to_single_value(),
            _ => self,
        }
    }
}

impl Default for LuaValue {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<bool> for LuaValue {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
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

impl From<TableValue> for LuaValue {
    fn from(value: TableValue) -> Self {
        Self::Table(value)
    }
}

impl From<FunctionValue> for LuaValue {
    fn from(function: FunctionValue) -> Self {
        Self::Function2(function)
    }
}

impl From<EngineFunction> for LuaValue {
    fn from(function: EngineFunction) -> Self {
        Self::Function2(FunctionValue::Engine(function))
    }
}

impl From<TupleValue> for LuaValue {
    fn from(tuple: TupleValue) -> Self {
        Self::Tuple(tuple)
    }
}

impl FromIterator<LuaValue> for LuaValue {
    fn from_iter<T: IntoIterator<Item = LuaValue>>(iter: T) -> Self {
        Self::Tuple(TupleValue::new(iter.into_iter().collect()))
    }
}

impl From<Vec<LuaValue>> for LuaValue {
    fn from(values: Vec<LuaValue>) -> Self {
        Self::Tuple(TupleValue::new(values))
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

    #[test]
    fn table_value_is_truthy() {
        assert!(LuaValue::empty_table().is_truthy().unwrap());
    }

    mod number_coercion {
        use super::*;

        macro_rules! number_coercion {
            ($($name:ident ($string:literal) => $result:expr),*) => {
                $(
                    #[test]
                    fn $name() {
                        assert_eq!(
                            LuaValue::String($string.into()).number_coercion(),
                            LuaValue::Number($result)
                        );
                    }
                )*
            };
        }

        macro_rules! no_number_coercion {
            ($($name:ident ($string:literal)),*) => {
                $(
                    #[test]
                    fn $name() {
                        assert_eq!(
                            LuaValue::String($string.into()).number_coercion(),
                            LuaValue::String($string.into())
                        );
                    }
                )*
            };
        }

        number_coercion!(
            zero("0") => 0.0,
            integer("12") => 12.0,
            integer_with_leading_zeros("00012") => 12.0,
            integer_with_ending_space("12   ") => 12.0,
            integer_with_leading_space("  123") => 123.0,
            integer_with_leading_tab("\t123") => 123.0,
            negative_integer("-3") => -3.0,
            hex_zero("0x0") => 0.0,
            hex_integer("0xA") => 10.0,
            negative_hex_integer("-0xA") => -10.0,
            float("0.5") => 0.5,
            negative_float("-0.5") => -0.5,
            float_starting_with_dot(".5") => 0.5
        );

        no_number_coercion!(
            letter_suffix("123a"),
            hex_prefix("0x"),
            space_between_minus("- 1"),
            two_seperated_digits(" 1 2")
        );
    }
}
