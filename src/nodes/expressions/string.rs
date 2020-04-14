use crate::lua_generator::{LuaGenerator, ToLua};

use std::str::CharIndices;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringExpression {
    value: String,
}

impl StringExpression {
    pub fn new(string: String) -> Option<Self> {
        let mut chars = string.chars();
        let value = match chars.next() {
            Some('"') | Some('\'') => string.get(1..string.len()-1).map(str::to_owned),
            Some('[') => chars.enumerate()
                    .find_map(|(indice, character)| if character == '[' { Some(indice) } else { None })
                    .and_then(|indice| {
                        let length = 2 + indice;
                        string.get(length..string.len()-length).map(str::to_owned)
                    }),
            _ => None,
        };

        value.map(|value| Self { value })
    }

    pub fn from_value<T: Into<String>>(value: T) -> Self {
        Self { value: value.into() }
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn is_multiline(&self) -> bool {
        self.value.find("\n").is_some()
    }

    pub fn has_single_quote(&self) -> bool {
        self.find_not_escaped('\'').is_some()
    }

    pub fn has_double_quote(&self) -> bool {
        self.find_not_escaped('"').is_some()
    }

    fn find_not_escaped(&self, pattern: char) -> Option<usize> {
        self.find_not_escaped_from(pattern, &mut self.value.char_indices())
    }

    fn find_not_escaped_from(&self, pattern: char, chars: &mut CharIndices) -> Option<usize> {
        let mut escaped = false;
        chars.find_map(|(index, character)| {
            if escaped {
                escaped = false;
                None
            } else {
                match character {
                    '\\' => {
                        escaped = true;
                        None
                    }
                    value => if value == pattern {
                        Some(index)
                    } else {
                        None
                    },
                }
            }
        })
    }
}

fn break_long_string(last_str: &str) -> bool {
    if let Some(last_char) = last_str.chars().last() {
        last_char == '['
    } else {
        false
    }
}

impl ToLua for StringExpression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        if self.is_multiline() {
            let mut i = 0;
            let mut equals = "=".repeat(i);

            loop {
                if self.value.find(&format!("]{}]", equals)).is_none() {
                    break
                } else {
                    i += 1;
                    equals = "=".repeat(i);
                };
            }

            generator.push_str_and_break_if(
                &format!("[{}[{}]{}]", equals, self.value, equals),
                break_long_string
            );

        } else {
            let string = if self.has_single_quote() {
                if self.has_double_quote() {
                    let mut total_escaped = 0;
                    let mut escaped_string = self.value.clone();

                    let mut chars = self.value.char_indices();

                    while let Some(unescaped_index) = self.find_not_escaped_from('\'', &mut chars) {
                        escaped_string.insert(unescaped_index + total_escaped, '\\');
                        total_escaped += 1;
                    }

                    format!("'{}'", escaped_string)
                } else {
                    format!("\"{}\"", self.value)
                }
            } else {
                format!("'{}'", self.value)
            };

            generator.push_str(&string);
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_removes_double_quotes() {
        let string = StringExpression::new(r#""hello""#.to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_single_quotes() {
        let string = StringExpression::new("'hello'".to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets() {
        let string = StringExpression::new("[[hello]]".to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets_with_one_equals() {
        let string = StringExpression::new("[=[hello]=]".to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets_with_multiple_equals() {
        let string = StringExpression::new("[==[hello]==]".to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn has_single_quote_is_false_if_no_single_quotes() {
        let string = StringExpression::from_value("hello");

        assert_eq!(string.has_single_quote(), false);
    }

    #[test]
    fn has_single_quote_is_true_if_unescaped_single_quotes() {
        let string = StringExpression::from_value("don't");

        assert_eq!(string.has_single_quote(), true);
    }

    #[test]
    fn has_single_quote_is_true_if_unescaped_single_quotes_with_escaped_backslash() {
        let string = StringExpression::from_value(r#"don\\'t"#);

        assert_eq!(string.has_single_quote(), true);
    }

    #[test]
    fn has_single_quote_is_false_if_escaped_single_quotes() {
        let string = StringExpression::from_value(r#"don\'t"#);

        assert_eq!(string.has_single_quote(), false);
    }

    #[test]
    fn has_double_quote_is_false_if_no_double_quotes() {
        let string = StringExpression::from_value("hello");

        assert_eq!(string.has_double_quote(), false);
    }

    #[test]
    fn has_double_quote_is_true_if_unescaped_double_quotes() {
        let string = StringExpression::from_value(r#"Say: "Hi!""#);

        assert_eq!(string.has_double_quote(), true);
    }

    #[test]
    fn has_double_quote_is_false_if_escaped_double_quotes() {
        let string = StringExpression::from_value(r#"hel\"o"#);

        assert_eq!(string.has_double_quote(), false);
    }

    #[test]
    fn generate_string_without_quotes_uses_single_quotes() {
        let output = StringExpression::from_value("hello").to_lua_string();

        assert_eq!(output, "'hello'");
    }

    #[test]
    fn generate_string_with_single_quotes_uses_double_quotes() {
        let output = StringExpression::from_value("don\'t").to_lua_string();

        assert_eq!(output, r#""don't""#);
    }

    #[test]
    fn generate_string_with_single_and_double_quotes_escapes_single_quotes() {
        let output = StringExpression::from_value(r#"Say: "Don't""#).to_lua_string();

        assert_eq!(output, r#"'Say: "Don\'t"'"#);
    }

    #[test]
    fn break_long_string_if_last_char_is_bracket() {
        let mut generator = LuaGenerator::default();
        generator.push_char('[');
        StringExpression::from_value("\n").to_lua(&mut generator);

        assert_eq!(generator.into_string(), "[ [[\n]]");
    }

    mod snapshot {
        use super::*;

        use insta::assert_snapshot;

        macro_rules! do_snapshots {
            ($($name:ident => $string:literal),+) => {
                $(
                    #[test]
                    fn $name() {
                        assert_snapshot!(
                            stringify!($name),
                            StringExpression::from_value(r#"Say: "Don't""#).to_lua_string()
                        );
                    }
                )+
            };
        }

        do_snapshots!(
            single_quotes => r#"hello"#,
            double_quotes => r#"I'm cool"#,
            escape_single_quotes => r#"Say: "Don't""#
        );
    }
}
