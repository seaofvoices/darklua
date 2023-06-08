use std::{
    iter::Peekable,
    str::{CharIndices, Chars},
};

use crate::nodes::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringExpression {
    value: String,
    token: Option<Token>,
}

impl StringExpression {
    pub fn new(string: &str) -> Option<Self> {
        if string.starts_with('[') {
            return string
                .chars()
                .skip(1)
                .enumerate()
                .find_map(|(indice, character)| if character == '[' { Some(indice) } else { None })
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
                    string.get(start..string.len() - length).map(str::to_owned)
                })
                .map(|value| Self { value, token: None });
        }

        let mut chars = string.chars().peekable();
        chars.next();
        chars.next_back();
        let mut value = String::new();
        value.reserve(string.as_bytes().len());

        while let Some(char) = chars.next() {
            if char == '\\' {
                if let Some(next_char) = chars.next() {
                    let escaped = match next_char {
                        'n' | '\n' => Some('\n'),
                        '"' => Some('"'),
                        '\'' => Some('\''),
                        '\\' => Some('\\'),
                        't' => Some('\t'),
                        'a' => Some('\u{7}'),
                        'b' => Some('\u{8}'),
                        'v' => Some('\u{B}'),
                        'f' => Some('\u{C}'),
                        'r' => Some('\r'),
                        first_digit if first_digit.is_ascii_digit() => {
                            let number = read_number(&mut chars, Some(first_digit), 10, 3);

                            if number < 256 {
                                value.push(number as u8 as char);
                            } else {
                                // malformed string sequence: cannot escape ascii character
                                // with a number >= 256
                                return None;
                            }

                            None
                        }
                        'x' => {
                            if let (Some(first_digit), Some(second_digit)) = (
                                chars.next().filter(char::is_ascii_hexdigit),
                                chars.next().filter(char::is_ascii_hexdigit),
                            ) {
                                let number = 16 * first_digit.to_digit(16).unwrap()
                                    + second_digit.to_digit(16).unwrap();

                                if number < 256 {
                                    value.push(number as u8 as char);
                                } else {
                                    unreachable!(
                                        "malformed string sequence: cannot escape ascii character >= 256",
                                    );
                                }
                            } else {
                                // malformed string sequence: missing one or both hex digits
                                return None;
                            }
                            None
                        }
                        'u' => {
                            if !contains(&chars.next(), &'{') {
                                // malformed string sequence: missing opening curly brace `{`
                                return None;
                            }

                            let number = read_number(&mut chars, None, 16, 8);

                            if !contains(&chars.next(), &'}') {
                                // malformed string sequence: missing closing curly brace `}`
                                return None;
                            }

                            if number > 0x10FFFF {
                                // malformed string sequence: invalid unicode value (too large)
                                return None;
                            }

                            value.push(
                                char::from_u32(number).expect("unable to convert u32 to char"),
                            );

                            None
                        }
                        'z' => {
                            while chars
                                .peek()
                                .filter(|char| char.is_ascii_whitespace())
                                .is_some()
                            {
                                chars.next();
                            }
                            None
                        }
                        _ => {
                            // malformed string sequence: invalid character after `\`
                            return None;
                        }
                    };

                    if let Some(escaped) = escaped {
                        value.push(escaped);
                    }
                } else {
                    // malformed string sequence: string ended after `\`
                    return None;
                }
            } else {
                value.push(char);
            }
        }

        value.shrink_to_fit();

        Some(Self::from_value(value))
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

    pub fn clear_comments(&mut self) {
        if let Some(token) = &mut self.token {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(token) = &mut self.token {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(token) = &mut self.token {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(token) = &mut self.token {
            token.shift_token_line(amount);
        }
    }
}

fn read_number(
    chars: &mut Peekable<Chars>,
    first_digit: Option<char>,
    radix: u32,
    max: usize,
) -> u32 {
    let filter = match radix {
        10 => char::is_ascii_digit,
        16 => char::is_ascii_hexdigit,
        _ => panic!("unsupported radix {}", radix),
    };
    let mut amount = first_digit
        .map(|char| char.to_digit(radix).unwrap())
        .unwrap_or(0);
    let mut iteration_count: usize = first_digit.is_some().into();

    while let Some(next_digit) = chars.peek().cloned().filter(filter) {
        chars.next();

        amount = amount * radix + next_digit.to_digit(radix).unwrap();
        iteration_count += 1;

        if iteration_count >= max {
            break;
        }
    }

    amount
}

fn contains<T, U>(option: &Option<T>, x: &U) -> bool
where
    U: PartialEq<T>,
{
    match option {
        Some(y) => x == y,
        None => false,
    }
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
                        assert!(StringExpression::new(&quoted).is_none());
                    }
                )*
            }

            mod double_quoted_failures {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let quoted = format!("\"{}\"", $input);
                        assert!(StringExpression::new(&quoted).is_none());
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
        invalid_escape => "\\o",
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
        let string = StringExpression::from_value(r#"don\\'t"#);

        assert!(string.has_single_quote());
    }

    #[test]
    fn has_single_quote_is_false_if_escaped_single_quotes() {
        let string = StringExpression::from_value(r#"don\'t"#);

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
