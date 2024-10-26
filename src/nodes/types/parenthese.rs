use crate::nodes::Token;

use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentheseType {
    inner_type: Box<Type>,
    tokens: Option<ParentheseTypeTokens>,
}

impl ParentheseType {
    pub fn new(r#type: impl Into<Type>) -> Self {
        Self {
            inner_type: Box::new(r#type.into()),
            tokens: None,
        }
    }

    #[inline]
    pub fn get_inner_type(&self) -> &Type {
        &self.inner_type
    }

    #[inline]
    pub fn into_inner_type(self) -> Type {
        *self.inner_type
    }

    #[inline]
    pub fn mutate_inner_type(&mut self) -> &mut Type {
        &mut self.inner_type
    }

    pub fn with_tokens(mut self, tokens: ParentheseTypeTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: ParentheseTypeTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&ParentheseTypeTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut ParentheseTypeTokens> {
        self.tokens.as_mut()
    }

    super::impl_token_fns!(iter = [tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentheseTypeTokens {
    pub left_parenthese: Token,
    pub right_parenthese: Token,
}

impl ParentheseTypeTokens {
    super::impl_token_fns!(target = [left_parenthese, right_parenthese]);
}
