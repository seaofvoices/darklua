use std::iter::FromIterator;

use crate::nodes::{StringError, Token};

use super::{string_utils, Expression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringSegment {
    value: String,
    token: Option<Token>,
}

impl StringSegment {
    pub fn new(value: impl AsRef<str>) -> Result<Self, StringError> {
        string_utils::read_escaped_string(value.as_ref().char_indices(), None).map(Self::from_value)
    }

    pub fn from_value(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    pub fn get_value(&self) -> &str {
        self.value.as_str()
    }

    fn append(&mut self, mut other: Self) {
        self.value.extend(other.value.drain(..));
        self.token = None;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueSegment {
    value: Expression,
    tokens: Option<ValueSegmentTokens>,
}

impl ValueSegment {
    pub fn new(value: impl Into<Expression>) -> Self {
        Self {
            value: value.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: ValueSegmentTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn set_tokens(&mut self, tokens: ValueSegmentTokens) {
        self.tokens = Some(tokens);
    }

    pub fn get_tokens(&self) -> Option<&ValueSegmentTokens> {
        self.tokens.as_ref()
    }

    pub fn get_expression(&self) -> &Expression {
        &self.value
    }

    pub fn mutate_expression(&mut self) -> &mut Expression {
        &mut self.value
    }

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueSegmentTokens {
    pub opening_brace: Token,
    pub closing_brace: Token,
}

impl ValueSegmentTokens {
    pub fn clear_comments(&mut self) {
        self.opening_brace.clear_comments();
        self.closing_brace.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_brace.clear_whitespaces();
        self.closing_brace.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_brace.replace_referenced_tokens(code);
        self.closing_brace.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.opening_brace.shift_token_line(amount);
        self.closing_brace.shift_token_line(amount);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterpolationSegment {
    String(StringSegment),
    Value(ValueSegment),
}

impl InterpolationSegment {
    pub fn clear_comments(&mut self) {
        match self {
            InterpolationSegment::String(segment) => segment.clear_comments(),
            InterpolationSegment::Value(segment) => segment.clear_comments(),
        }
    }

    pub fn clear_whitespaces(&mut self) {
        match self {
            InterpolationSegment::String(segment) => segment.clear_whitespaces(),
            InterpolationSegment::Value(segment) => segment.clear_whitespaces(),
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        match self {
            InterpolationSegment::String(segment) => segment.replace_referenced_tokens(code),
            InterpolationSegment::Value(segment) => segment.replace_referenced_tokens(code),
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        match self {
            InterpolationSegment::String(segment) => segment.shift_token_line(amount),
            InterpolationSegment::Value(segment) => segment.shift_token_line(amount),
        }
    }
}

impl From<StringSegment> for InterpolationSegment {
    fn from(segment: StringSegment) -> Self {
        Self::String(segment)
    }
}

impl From<ValueSegment> for InterpolationSegment {
    fn from(segment: ValueSegment) -> Self {
        Self::Value(segment)
    }
}

impl<T: Into<Expression>> From<T> for InterpolationSegment {
    fn from(value: T) -> Self {
        Self::Value(ValueSegment::new(value.into()))
    }
}

impl From<&str> for InterpolationSegment {
    fn from(string: &str) -> Self {
        Self::String(StringSegment::from_value(string))
    }
}

impl From<&String> for InterpolationSegment {
    fn from(string: &String) -> Self {
        Self::String(StringSegment::from_value(string))
    }
}

impl From<String> for InterpolationSegment {
    fn from(string: String) -> Self {
        Self::String(StringSegment::from_value(string))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpolatedStringExpression {
    segments: Vec<InterpolationSegment>,
    tokens: Option<InterpolatedStringTokens>,
}

impl InterpolatedStringExpression {
    pub fn new(segments: Vec<InterpolationSegment>) -> Self {
        Self {
            segments,
            tokens: None,
        }
    }

    pub fn empty() -> Self {
        Self::new(Vec::default())
    }

    pub fn with_segment(mut self, segment: impl Into<InterpolationSegment>) -> Self {
        self.push_segment(segment);
        self
    }

    pub fn with_tokens(mut self, tokens: InterpolatedStringTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn get_tokens(&self) -> Option<&InterpolatedStringTokens> {
        self.tokens.as_ref()
    }

    pub fn set_tokens(&mut self, tokens: InterpolatedStringTokens) {
        self.tokens = Some(tokens);
    }

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
        for segment in &mut self.segments {
            segment.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
        for segment in &mut self.segments {
            segment.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
        for segment in &mut self.segments {
            segment.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
        for segment in &mut self.segments {
            segment.shift_token_line(amount);
        }
    }

    pub fn iter_segments(&self) -> impl Iterator<Item = &InterpolationSegment> {
        self.segments.iter()
    }

    pub fn iter_mut_segments(&mut self) -> impl Iterator<Item = &mut InterpolationSegment> {
        self.segments.iter_mut()
    }

    pub fn push_segment(&mut self, segment: impl Into<InterpolationSegment>) {
        let new_segment = segment.into();
        match new_segment {
            InterpolationSegment::String(string_segment) => {
                if string_segment.get_value().is_empty() {
                    return;
                }
                if let Some(InterpolationSegment::String(last)) = self.segments.last_mut() {
                    last.append(string_segment);
                } else {
                    self.segments.push(string_segment.into());
                }
            }
            InterpolationSegment::Value(_) => {
                self.segments.push(new_segment);
            }
        }
    }
}

impl FromIterator<InterpolationSegment> for InterpolatedStringExpression {
    fn from_iter<T: IntoIterator<Item = InterpolationSegment>>(iter: T) -> Self {
        Self {
            segments: iter.into_iter().collect(),
            tokens: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpolatedStringTokens {
    pub opening_tick: Token,
    pub closing_tick: Token,
}

impl InterpolatedStringTokens {
    pub fn clear_comments(&mut self) {
        self.opening_tick.clear_comments();
        self.closing_tick.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_tick.clear_whitespaces();
        self.closing_tick.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_tick.replace_referenced_tokens(code);
        self.closing_tick.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.opening_tick.shift_token_line(amount);
        self.closing_tick.shift_token_line(amount);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push_segment_with_empty_string_does_not_mutate() {
        let mut string = InterpolatedStringExpression::empty();
        string.push_segment("");

        pretty_assertions::assert_eq!(string, InterpolatedStringExpression::empty());
    }
}
