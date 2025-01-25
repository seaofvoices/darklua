use std::str::CharIndices;

use crate::nodes::{StringError, Token};

use super::string_utils;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringExpression {
    value: String,
    token: Option<Token>,
}

impl StringExpression {
    pub fn new(string: &str) -> Result<Self, StringError> {
        if string.starts_with('[') {
            return string
                .chars()
                .skip(1)
                .enumerate()
                .find_map(|(indice, character)| if character == '[' { Some(indice) } else { None })
                .ok_or_else(|| StringError::invalid("unable to find `[` delimiter"))
                .and_then(|indice| {
                    let length = 2 + indice;
                    let start = if string
                        .get(length..length + 1)
                        .filter(|char| char == &"\n")
                        .is_some()
                    {
                        length + 1
                    } else {
                        length
                    };
                    string
                        .get(start..string.len() - length)
                        .map(str::to_owned)
                        .ok_or_else(|| StringError::invalid(""))
                })
                .map(Self::from_value);
        }

        let mut chars = string.char_indices();

        match (chars.next(), chars.next_back()) {
            (Some((_, first_char)), Some((_, last_char))) if first_char == last_char => {
                string_utils::read_escaped_string(chars, Some(string.len())).map(Self::from_value)
            }
            (None, None) | (None, Some(_)) | (Some(_), None) => {
                Err(StringError::invalid("missing quotes"))
            }
            (Some(_), Some(_)) => Err(StringError::invalid("quotes do not match")),
        }
    }

    pub fn empty() -> Self {
        Self {
            value: "".to_owned(),
            token: None,
        }
    }

    pub fn from_value<T: Into<String>>(value: T) -> Self {
        Self {
            value: value.into(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    #[inline]
    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    #[inline]
    pub fn get_token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    #[inline]
    pub fn get_value(&self) -> &str {
        &self.value
    }

    #[inline]
    pub fn into_value(self) -> String {
        self.value
    }

    pub fn is_multiline(&self) -> bool {
        self.value.contains('\n')
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
                    value => {
                        if value == pattern {
                            Some(index)
                        } else {
                            None
                        }
                    }
                }
            }
        })
    }

    super::impl_token_fns!(iter = [token]);
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_quoted {
        ($($name:ident($input:literal) => $value:literal),* $(,)?) => {
            mod single_quoted {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let quoted = format!("'{}'", $input);
                        assert_eq!(
                            StringExpression::new(&quoted)
                                .expect("unable to parse string")
                                .get_value(),
                            StringExpression::from_value($value).get_value(),
                        );
                    }
                )*
            }

            mod double_quoted {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let quoted = format!("\"{}\"", $input);
                        assert_eq!(
                            StringExpression::new(&quoted)
                                .expect("unable to parse string")
                                .get_value(),
                            StringExpression::from_value($value).get_value(),
                        );
                    }
                )*
            }
        };
    }

    test_quoted!(
        empty("") => "",
        hello("hello") => "hello",
        escaped_new_line("\\n") => "\n",
        escaped_tab("\\t") => "\t",
        escaped_backslash("\\\\") => "\\",
        escaped_carriage_return("\\r") => "\r",
        escaped_bell("\\a") => "\u{7}",
        escaped_backspace("\\b") => "\u{8}",
        escaped_vertical_tab("\\v") => "\u{B}",
        escaped_form_feed("\\f") => "\u{C}",
        escaped_null("\\0") => "\0",
        escaped_two_digits("\\65") => "A",
        escaped_three_digits("\\123") => "{",
        escaped_null_hex("\\x00") => "\0",
        escaped_uppercase_a_hex("\\x41") => "A",
        escaped_tilde_hex_uppercase("\\x7E") => "~",
        escaped_tilde_hex_lowercase("\\x7e") => "~",
        skips_whitespaces_but_no_spaces("\\z") => "",
        skips_whitespaces("a\\z   \n\n   \\nb") => "a\nb",
        escaped_unicode_single_digit("\\u{0}") => "\0",
        escaped_unicode_two_hex_digits("\\u{AB}") => "\u{AB}",
        escaped_unicode_three_digit("\\u{123}") => "\u{123}",
        escaped_unicode_last_value("\\u{10FFFF}") => "\u{10FFFF}",
    );

    macro_rules! test_quoted_failures {
        ($($name:ident => $input:literal),* $(,)?) => {
            mod single_quoted_failures {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let quoted = format!("'{}'", $input);
                        assert!(StringExpression::new(&quoted).is_err());
                    }
                )*
            }

            mod double_quoted_failures {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let quoted = format!("\"{}\"", $input);
                        assert!(StringExpression::new(&quoted).is_err());
                    }
                )*
            }
        };
    }

    test_quoted_failures!(
        single_backslash => "\\",
        escaped_too_large_ascii => "\\256",
        escaped_too_large_unicode => "\\u{110000}",
        escaped_missing_opening_brace_unicode => "\\uAB",
        escaped_missing_closing_brace_unicode => "\\u{0p",
    );

    #[test]
    fn new_removes_double_quotes() {
        let string = StringExpression::new(r#""hello""#).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_single_quotes() {
        let string = StringExpression::new("'hello'").unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets() {
        let string = StringExpression::new("[[hello]]").unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets_and_skip_first_new_line() {
        let string = StringExpression::new("[[\nhello]]").unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets_with_one_equals() {
        let string = StringExpression::new("[=[hello]=]").unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets_with_multiple_equals() {
        let string = StringExpression::new("[==[hello]==]").unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_skip_invalid_escape_in_double_quoted_string() {
        let string = StringExpression::new("'\\oo'").unwrap();

        assert_eq!(string.get_value(), "oo");
    }

    #[test]
    fn new_skip_invalid_escape_in_single_quoted_string() {
        let string = StringExpression::new("\"\\oo\"").unwrap();

        assert_eq!(string.get_value(), "oo");
    }

    #[test]
    fn has_single_quote_is_false_if_no_single_quotes() {
        let string = StringExpression::from_value("hello");

        assert!(!string.has_single_quote());
    }

    #[test]
    fn has_single_quote_is_true_if_unescaped_single_quotes() {
        let string = StringExpression::from_value("don't");

        assert!(string.has_single_quote());
    }

    #[test]
    fn has_single_quote_is_true_if_unescaped_single_quotes_with_escaped_backslash() {
        let string = StringExpression::from_value(r"don\\'t");

        assert!(string.has_single_quote());
    }

    #[test]
    fn has_single_quote_is_false_if_escaped_single_quotes() {
        let string = StringExpression::from_value(r"don\'t");

        assert!(!string.has_single_quote());
    }

    #[test]
    fn has_double_quote_is_false_if_no_double_quotes() {
        let string = StringExpression::from_value("hello");

        assert!(!string.has_double_quote());
    }

    #[test]
    fn has_double_quote_is_true_if_unescaped_double_quotes() {
        let string = StringExpression::from_value(r#"Say: "Hi!""#);

        assert!(string.has_double_quote());
    }

    #[test]
    fn has_double_quote_is_false_if_escaped_double_quotes() {
        let string = StringExpression::from_value(r#"hel\"o"#);

        assert!(!string.has_double_quote());
    }
}
