use crate::nodes::{Identifier, Prefix, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldExpression {
    prefix: Prefix,
    field: Identifier,
    token: Option<Token>,
}

impl FieldExpression {
    pub fn new<IntoPrefix: Into<Prefix>, IntoIdentifier: Into<Identifier>>(
        prefix: IntoPrefix,
        field: IntoIdentifier,
    ) -> Self {
        Self {
            prefix: prefix.into(),
            field: field.into(),
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
    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    #[inline]
    pub fn get_field(&self) -> &Identifier {
        &self.field
    }

    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }

    pub fn clear_comments(&mut self) {
        self.field.clear_comments();
        if let Some(token) = &mut self.token {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.field.clear_whitespaces();
        if let Some(token) = &mut self.token {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.field.replace_referenced_tokens(code);
        if let Some(token) = &mut self.token {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.field.shift_token_line(amount);
        if let Some(token) = &mut self.token {
            token.shift_token_line(amount);
        }
    }
}
