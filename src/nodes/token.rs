use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Position {
    LineNumberReference {
        start: usize,
        end: usize,
        line_number: usize,
    },
    LineNumber {
        content: Cow<'static, str>,
        line_number: usize,
    },
    Any {
        content: Cow<'static, str>,
    },
}

impl Position {
    #[inline]
    pub fn line_number(content: impl Into<Cow<'static, str>>, line_number: usize) -> Position {
        Self::LineNumber {
            content: content.into(),
            line_number,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TriviaKind {
    Comment,
    Whitespace,
}

impl TriviaKind {
    pub fn at(self, start: usize, end: usize, line_number: usize) -> Trivia {
        Trivia {
            position: Position::LineNumberReference {
                start,
                end,
                line_number,
            },
            kind: self,
        }
    }

    pub fn with_content<IntoCowStr: Into<Cow<'static, str>>>(self, content: IntoCowStr) -> Trivia {
        Trivia {
            position: Position::Any {
                content: content.into(),
            },
            kind: self,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trivia {
    position: Position,
    kind: TriviaKind,
}

impl Trivia {
    pub fn read<'a: 'b, 'b>(&'a self, code: &'b str) -> &'b str {
        match &self.position {
            Position::LineNumberReference { start, end, .. } => {
                code.get(*start..*end).unwrap_or_else(|| {
                    panic!("unable to extract code from position: {} - {}", start, end);
                })
            }
            Position::LineNumber { content, .. } | Position::Any { content } => content,
        }
    }

    pub fn try_read(&self) -> Option<&str> {
        match &self.position {
            Position::LineNumberReference { .. } => None,
            Position::LineNumber { content, .. } | Position::Any { content } => Some(content),
        }
    }

    pub fn kind(&self) -> TriviaKind {
        self.kind.clone()
    }

    pub fn get_line_number(&self) -> Option<usize> {
        match &self.position {
            Position::LineNumber { line_number, .. }
            | Position::LineNumberReference { line_number, .. } => Some(*line_number),
            Position::Any { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    position: Position,
    leading_trivia: Vec<Trivia>,
    trailing_trivia: Vec<Trivia>,
}

impl Token {
    /// Creates a token where the position refers to the original code where
    /// the token was parsed with the line number where it starts.
    pub fn new_with_line(start: usize, end: usize, line_number: usize) -> Self {
        Self {
            position: Position::LineNumberReference {
                start,
                end,
                line_number,
            },
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }

    /// Creates a new token that is not contrained to any existing position.
    pub fn from_content<IntoCowStr: Into<Cow<'static, str>>>(content: IntoCowStr) -> Self {
        Self {
            position: Position::Any {
                content: content.into(),
            },
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }

    /// Creates a new token from a position.
    pub fn from_position(position: Position) -> Self {
        Self {
            position,
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }

    pub fn with_leading_trivia(mut self, trivia: Trivia) -> Self {
        self.leading_trivia.push(trivia);
        self
    }

    pub fn with_trailing_trivia(mut self, trivia: Trivia) -> Self {
        self.trailing_trivia.push(trivia);
        self
    }

    #[inline]
    pub fn has_trivia(&self) -> bool {
        !self.leading_trivia.is_empty() || !self.trailing_trivia.is_empty()
    }

    #[inline]
    pub fn push_leading_trivia(&mut self, trivia: Trivia) {
        self.leading_trivia.push(trivia);
    }

    #[inline]
    pub fn push_trailing_trivia(&mut self, trivia: Trivia) {
        self.trailing_trivia.push(trivia);
    }

    #[inline]
    pub fn iter_leading_trivia(&self) -> impl Iterator<Item = &Trivia> {
        self.leading_trivia.iter()
    }

    #[inline]
    pub fn iter_trailing_trivia(&self) -> impl Iterator<Item = &Trivia> {
        self.trailing_trivia.iter()
    }

    pub fn read<'a: 'b, 'b>(&'a self, code: &'b str) -> &'b str {
        match &self.position {
            Position::LineNumberReference { start, end, .. } => code
                .get(*start..*end)
                .expect("unable to extract code from position"),
            Position::LineNumber { content, .. } | Position::Any { content } => content,
        }
    }

    pub fn get_line_number(&self) -> Option<usize> {
        match &self.position {
            Position::LineNumber { line_number, .. }
            | Position::LineNumberReference { line_number, .. } => Some(*line_number),
            Position::Any { .. } => None,
        }
    }

    pub fn replace_with_content<IntoCowStr: Into<Cow<'static, str>>>(
        &mut self,
        content: IntoCowStr,
    ) {
        self.position = match &self.position {
            Position::LineNumber { line_number, .. }
            | Position::LineNumberReference { line_number, .. } => Position::LineNumber {
                line_number: *line_number,
                content: content.into(),
            },

            Position::Any { .. } => Position::Any {
                content: content.into(),
            },
        };
    }

    pub fn clear_comments(&mut self) {
        self.leading_trivia
            .retain(|trivia| trivia.kind() != TriviaKind::Comment);
        self.trailing_trivia
            .retain(|trivia| trivia.kind() != TriviaKind::Comment);
    }

    pub fn clear_whitespaces(&mut self) {
        self.leading_trivia
            .retain(|trivia| trivia.kind() != TriviaKind::Whitespace);
        self.trailing_trivia
            .retain(|trivia| trivia.kind() != TriviaKind::Whitespace);
    }

    pub(crate) fn filter_comments(&mut self, filter: impl Fn(&Trivia) -> bool) {
        self.leading_trivia
            .retain(|trivia| trivia.kind() != TriviaKind::Comment || filter(trivia));
        self.trailing_trivia
            .retain(|trivia| trivia.kind() != TriviaKind::Comment || filter(trivia));
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Position::LineNumberReference {
            start,
            end,
            line_number,
        } = self.position
        {
            self.position = Position::LineNumber {
                line_number,
                content: code
                    .get(start..end)
                    .expect("unable to extract code from position")
                    .to_owned()
                    .into(),
            }
        };
        for trivia in self
            .leading_trivia
            .iter_mut()
            .chain(self.trailing_trivia.iter_mut())
        {
            if let Position::LineNumberReference {
                start,
                end,
                line_number,
            } = trivia.position
            {
                trivia.position = Position::LineNumber {
                    line_number,
                    content: code
                        .get(start..end)
                        .expect("unable to extract code from position")
                        .to_owned()
                        .into(),
                }
            };
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: isize) {
        match &mut self.position {
            Position::LineNumberReference { line_number, .. }
            | Position::LineNumber { line_number, .. } => {
                *line_number = line_number.saturating_add_signed(amount);
            }
            Position::Any { .. } => {}
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_line_number_reference_token() {
        let code = "return true";
        let token = Token::new_with_line(7, 11, 1);

        assert_eq!("true", token.read(code));
    }

    #[test]
    fn read_any_position_token() {
        let token = Token::from_content("true");

        assert_eq!("true", token.read(""));
    }
}
