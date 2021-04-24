use crate::nodes::{Expression, StringExpression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEntry {
    Field(String, Expression),
    Index(Expression, Expression),
    Value(Expression),
}

impl<T: Into<Expression>> From<T> for TableEntry {
    fn from(expression: T) -> Self {
        Self::Value(expression.into())
    }
}

fn is_valid_identifier(string: &str) -> bool {
    if let Some(first_char) = string.chars().next() {
        if first_char.is_alphabetic() || first_char == '_' {
            string.chars()
                .all(|character| {
                    character.is_ascii_alphanumeric()
                })
        } else {
            false
        }
    } else {
        false
    }
}

impl TableEntry {
    /// Creates a table entry like `Foo = ...` or `["123"] = ...` from the content
    /// of the field string. It will wrap the field in the bracket syntax if necessary.
    pub fn from_potential_string_field<I: Into<String>, E: Into<Expression>>(
        field: I,
        expression: E,
    ) -> Self {
        let field = field.into();
        let expression = expression.into();

        if is_valid_identifier(&field) {
            Self::Field(field, expression)
        } else {
            Self::Index(
                StringExpression::from_value(field).into(),
                expression
            )
        }
    }

    /// Creates a table entry like `Foo = ...` or `["123"] = ...` from the content
    /// of the field string. It will wrap the field in the bracket syntax if necessary.
    pub fn from_dictionary_entry<T: Into<Expression>, U: Into<Expression>>(
        key: T,
        value: U,
    ) -> Self {
        let key = key.into();
        let value = value.into();

        match key {
            Expression::String(string) => {
                Self::from_potential_string_field(string.get_value(), value)
            }
            _ => {
                Self::Index(key, value)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableExpression {
    entries: Vec<TableEntry>,
}

impl TableExpression {
    pub fn new(entries: Vec<TableEntry>) -> Self {
        Self {
            entries,
        }
    }

    #[inline]
    pub fn get_entries(&self) -> &Vec<TableEntry> {
        &self.entries
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn mutate_entries(&mut self) -> &mut Vec<TableEntry> {
        &mut self.entries
    }
}

impl Default for TableExpression {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod from_potential_string_field {
        use super::*;

        const SOME_EXPRESSION: Expression = Expression::True;

        macro_rules! test_index_entries {
            ($($name:ident ($string:literal)),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let result = TableEntry::from_potential_string_field(
                            $string,
                            SOME_EXPRESSION,
                        );
                        assert_eq!(result, TableEntry::Index(
                            StringExpression::from_value($string).into(),
                            SOME_EXPRESSION,
                        ))
                    }
                )*
            };
        }

        macro_rules! test_field_entries {
            ($($name:ident ($string:literal)),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let result = TableEntry::from_potential_string_field(
                            $string,
                            SOME_EXPRESSION,
                        );
                        assert_eq!(result, TableEntry::Field(
                            $string.to_owned(),
                            SOME_EXPRESSION,
                        ))
                    }
                )*
            };
        }

        test_field_entries!(
            letters_only("foo"),
            ends_with_digit("foo7"),
        );

        test_index_entries!(
            empty_string(""),
            starts_with_a_digit("1foo"),
            starts_with_a_space(" foo"),
            ends_with_a_space("foo "),
            has_a_space("foo bar"),
        );
    }
}