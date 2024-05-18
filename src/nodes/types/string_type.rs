use crate::nodes::{StringError, StringExpression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringType {
    value: StringExpression,
}

impl StringType {
    pub fn new(string: &str) -> Result<Self, StringError> {
        StringExpression::new(string).map(|value| Self { value })
    }

    pub fn empty() -> Self {
        Self {
            value: StringExpression::empty(),
        }
    }

    pub fn from_value<T: Into<String>>(value: T) -> Self {
        Self {
            value: StringExpression::from_value(value.into()),
        }
    }

    pub fn with_token(mut self, token: Token) -> Self {
        self.value.set_token(token);
        self
    }

    #[inline]
    pub fn set_token(&mut self, token: Token) {
        self.value.set_token(token);
    }

    #[inline]
    pub fn get_token(&self) -> Option<&Token> {
        self.value.get_token()
    }

    #[inline]
    pub fn get_value(&self) -> &str {
        self.value.get_value()
    }

    #[inline]
    pub fn into_value(self) -> String {
        self.value.into_value()
    }

    #[inline]
    pub fn is_multiline(&self) -> bool {
        self.value.is_multiline()
    }

    #[inline]
    pub fn has_single_quote(&self) -> bool {
        self.value.has_single_quote()
    }

    #[inline]
    pub fn has_double_quote(&self) -> bool {
        self.value.has_double_quote()
    }

    super::impl_token_fns!(target = [value]);
}
