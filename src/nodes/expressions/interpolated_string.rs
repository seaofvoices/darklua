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

    pub fn get_value(&self) -> &str {
        self.value.as_str()
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

    pub fn get_expression(&self) -> &Expression {
        &self.value
    }

    pub fn mutate_expression(&mut self) -> &mut Expression {
        &mut self.value
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

impl From<Expression> for InterpolationSegment {
    fn from(value: Expression) -> Self {
        Self::Value(ValueSegment::new(value))
    }
}

impl<T: AsRef<str>> From<T> for InterpolationSegment {
    fn from(string: T) -> Self {
        Self::String(StringSegment::from_value(string.as_ref()))
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
        self.segments.push(segment.into());
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
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub fn iter_segments(&self) -> impl Iterator<Item = &InterpolationSegment> {
        self.segments.iter()
    }

    pub fn iter_mut_segments(&mut self) -> impl Iterator<Item = &mut InterpolationSegment> {
        self.segments.iter_mut()
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
}
