use std::{fmt, iter::Peekable, str::CharIndices};

#[derive(Debug, Clone)]
enum StringErrorKind {
    Invalid { message: String },
    MalformedEscapeSequence { position: usize, message: String },
}

#[derive(Debug, Clone)]
pub struct StringError {
    kind: StringErrorKind,
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            StringErrorKind::Invalid { message } => {
                write!(f, "invalid string: {}", message)
            }
            StringErrorKind::MalformedEscapeSequence { position, message } => {
                write!(f, "malformed escape sequence at {}: {}", position, message)
            }
        }
    }
}

impl StringError {
    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self {
            kind: StringErrorKind::Invalid {
                message: message.into(),
            },
        }
    }
    pub(crate) fn malformed_escape_sequence(position: usize, message: impl Into<String>) -> Self {
        Self {
            kind: StringErrorKind::MalformedEscapeSequence {
                position,
                message: message.into(),
            },
        }
    }
}

pub(crate) fn read_escaped_string(
    chars: CharIndices,
    reserve_size: Option<usize>,
) -> Result<String, StringError> {
    let mut chars = chars.peekable();

    let mut value = String::new();
    if let Some(reserve_size) = reserve_size {
        value.reserve(reserve_size);
    }

    while let Some((position, char)) = chars.next() {
        if char == '\\' {
            if let Some((_, next_char)) = chars.next() {
                match next_char {
                    '\n' | '"' | '\'' | '\\' => value.push(next_char),
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'a' => value.push('\u{7}'),
                    'b' => value.push('\u{8}'),
                    'v' => value.push('\u{B}'),
                    'f' => value.push('\u{C}'),
                    'r' => value.push('\r'),
                    first_digit if first_digit.is_ascii_digit() => {
                        let number = read_number(&mut chars, Some(first_digit), 10, 3);

                        if number < 256 {
                            value.push(number as u8 as char);
                        } else {
                            return Err(StringError::malformed_escape_sequence(
                                position,
                                "cannot escape ascii character greater than 256",
                            ));
                        }
                    }
                    'x' => {
                        if let (Some(first_digit), Some(second_digit)) = (
                            chars.next().map(|(_, c)| c).filter(char::is_ascii_hexdigit),
                            chars.next().map(|(_, c)| c).filter(char::is_ascii_hexdigit),
                        ) {
                            let number = 16 * first_digit.to_digit(16).unwrap()
                                + second_digit.to_digit(16).unwrap();

                            if number < 256 {
                                value.push(number as u8 as char);
                            } else {
                                return Err(StringError::malformed_escape_sequence(
                                    position,
                                    "cannot escape ascii character greater than 256",
                                ));
                            }
                        } else {
                            return Err(StringError::malformed_escape_sequence(
                                position,
                                "exactly two hexadecimal digit expected",
                            ));
                        }
                    }
                    'u' => {
                        if !contains(&chars.next().map(|(_, c)| c), &'{') {
                            return Err(StringError::malformed_escape_sequence(
                                position,
                                "expected opening curly brace",
                            ));
                        }

                        let number = read_number(&mut chars, None, 16, 8);

                        if !contains(&chars.next().map(|(_, c)| c), &'}') {
                            return Err(StringError::malformed_escape_sequence(
                                position,
                                "expected closing curly brace",
                            ));
                        }

                        if number > 0x10FFFF {
                            return Err(StringError::malformed_escape_sequence(
                                position,
                                "invalid unicode value",
                            ));
                        }

                        value.push(char::from_u32(number).expect("unable to convert u32 to char"));
                    }
                    'z' => {
                        while chars
                            .peek()
                            .filter(|(_, char)| char.is_ascii_whitespace())
                            .is_some()
                        {
                            chars.next();
                        }
                    }
                    _ => {
                        // an invalid escape does not error: it simply skips the backslash
                        value.push(next_char);
                    }
                }
            } else {
                return Err(StringError::malformed_escape_sequence(
                    position,
                    "string ended after '\\'",
                ));
            }
        } else {
            value.push(char);
        }
    }

    value.shrink_to_fit();

    Ok(value)
}

fn read_number(
    chars: &mut Peekable<CharIndices>,
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

    while let Some(next_digit) = chars.peek().map(|(_, c)| *c).filter(filter) {
        chars.next();

        amount = amount * radix + next_digit.to_digit(radix).unwrap();
        iteration_count += 1;

        if iteration_count >= max {
            break;
        }
    }

    amount
}

#[inline]
fn contains<T, U>(option: &Option<T>, x: &U) -> bool
where
    U: PartialEq<T>,
{
    match option {
        Some(y) => x == y,
        None => false,
    }
}
