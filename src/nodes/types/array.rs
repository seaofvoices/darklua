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
pub struct ArrayTypeTokens {
    pub opening_brace: Token,
    pub closing_brace: Token,
}

impl ArrayTypeTokens {
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
