use std::iter::FromIterator;

use super::LuaValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupleValue {
    values: Vec<LuaValue>,
}

impl TupleValue {
    pub fn new(values: Vec<LuaValue>) -> Self {
        Self { values }
    }

    pub fn empty() -> Self {
        Self { values: Vec::new() }
    }

    pub fn singleton<T: Into<LuaValue>>(value: T) -> Self {
        Self {
            values: vec![value.into()],
        }
    }

    pub fn push<IntoLuaValue: Into<LuaValue>>(&mut self, value: IntoLuaValue) {
        let value = value.into();
        match value {
            LuaValue::Tuple(tuple) => {
                self.values.extend(tuple);
            }
            LuaValue::False
            | LuaValue::Function(_)
            | LuaValue::Nil
            | LuaValue::Number(_)
            | LuaValue::String(_)
            | LuaValue::Table(_)
            | LuaValue::TableRef(_)
            | LuaValue::True
            | LuaValue::Unknown => {
                self.values.push(value);
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn as_single_value(&self) -> &LuaValue {
        self.values.first().unwrap_or(&LuaValue::Nil)
    }

    pub fn coerce_to_single_value(self) -> LuaValue {
        self.values.into_iter().next().unwrap_or(LuaValue::Nil)
    }

    pub fn flatten(self) -> Self {
        let last_index = self.len().saturating_sub(1);
        let mut new_values = Vec::new();

        for (i, value) in self.values.into_iter().enumerate() {
            if i == last_index {
                match value {
                    LuaValue::Tuple(tuple) => {
                        new_values.extend(tuple);
                    }
                    _ => {
                        new_values.push(value);
                    }
                }
            } else {
                new_values.push(value.coerce_to_single_value());
            }
        }
        Self { values: new_values }
    }

    pub fn iter(&self) -> impl Iterator<Item = &LuaValue> {
        self.values.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut LuaValue> {
        self.values.iter_mut()
    }
}

impl From<LuaValue> for TupleValue {
    fn from(value: LuaValue) -> Self {
        match value {
            LuaValue::Tuple(tuple) => tuple,
            _ => Self {
                values: vec![value],
            },
        }
    }
}

impl FromIterator<LuaValue> for TupleValue {
    fn from_iter<T: IntoIterator<Item = LuaValue>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for TupleValue {
    type Item = LuaValue;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

macro_rules! drop_first {
    ($_drop:tt, $sub:ty) => {
        $sub
    };
}

macro_rules! impl_from_tuples {
    ($( $name:ident, )+) => {
        impl From<( $( drop_first!($name, LuaValue), )+ )> for TupleValue {
            fn from(($( $name, )+): ( $( drop_first!($name, LuaValue), )+ )) -> Self {
                Self {
                    values: vec![$( $name, )+],
                }
            }
        }
    };
}

impl_from_tuples!(a,);
impl_from_tuples!(a, b,);
impl_from_tuples!(a, b, c,);
impl_from_tuples!(a, b, c, d,);
impl_from_tuples!(a, b, c, d, e,);
impl_from_tuples!(a, b, c, d, e, f,);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_luavalue_tuple_variant_return_same_tuple() {
        pretty_assertions::assert_eq!(
            TupleValue::from(LuaValue::Tuple(TupleValue::empty())),
            TupleValue::empty(),
        );
    }

    #[test]
    fn from_unknown_luavalue_variant_return_tuple_with_unknown() {
        pretty_assertions::assert_eq!(
            TupleValue::from(LuaValue::Unknown),
            TupleValue::singleton(LuaValue::Unknown),
        );
    }

    mod flatten {
        use super::*;

        macro_rules! test_flatten {
            ($($name:ident ($expect:expr) => $value:expr),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        pretty_assertions::assert_eq!(
                            TupleValue::from($expect).flatten(),
                            TupleValue::from($value),
                        )
                    }
                )*
            }
        }

        test_flatten!(
            empty_tuple(TupleValue::empty()) => TupleValue::empty(),
            singleton_true_tuple(TupleValue::singleton(true)) => TupleValue::singleton(true),
            tuple_with_two_values((true.into(), false.into())) => (true.into(), false.into()),
            first_is_singleton_tuple((LuaValue::from(TupleValue::singleton(true)), false.into()))
                => (true.into(), false.into()),
            first_is_empty_tuple((LuaValue::from(TupleValue::empty()), false.into()))
                => (LuaValue::Nil, false.into()),
            first_has_multiple_values((LuaValue::from(vec![true.into(), false.into()]), false.into()))
                => (true.into(), false.into()),
            second_has_multiple_values((false.into(), LuaValue::from(vec![true.into(), false.into()])))
                => (false.into(), true.into(), false.into()),
        );
    }
}
