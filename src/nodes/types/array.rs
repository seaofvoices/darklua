use crate::nodes::Token;

use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayType {
    inner_type: Box<Type>,
    tokens: Option<ArrayTypeTokens>,
}

impl ArrayType {
    pub fn new(element_type: impl Into<Type>) -> Self {
        Self {
            inner_type: Box::new(element_type.into()),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: ArrayTypeTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: ArrayTypeTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&ArrayTypeTokens> {
        self.tokens.as_ref()
    }

    pub fn get_element_type(&self) -> &Type {
        &self.inner_type
    }

    pub fn mutate_element_type(&mut self) -> &mut Type {
        &mut self.inner_type
    }

    super::impl_token_fns!(iter = [tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayTypeTokens {
    pub opening_brace: Token,
    pub closing_brace: Token,
}

impl ArrayTypeTokens {
    super::impl_token_fns!(target = [opening_brace, closing_brace]);
}
