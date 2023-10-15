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
pub struct ParentheseTypeTokens {
    pub left_parenthese: Token,
    pub right_parenthese: Token,
}

impl ParentheseTypeTokens {
    pub fn clear_comments(&mut self) {
        self.left_parenthese.clear_comments();
        self.right_parenthese.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.left_parenthese.clear_whitespaces();
        self.right_parenthese.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.left_parenthese.replace_referenced_tokens(code);
        self.right_parenthese.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.left_parenthese.shift_token_line(amount);
        self.right_parenthese.shift_token_line(amount);
    }
}
