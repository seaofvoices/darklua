use crate::nodes::{Identifier, Token};

use super::TypeName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeField {
    namespace: Identifier,
    name: TypeName,
    token: Option<Token>,
}

impl TypeField {
    pub fn new(namespace: impl Into<Identifier>, type_name: TypeName) -> Self {
        Self {
            namespace: namespace.into(),
            name: type_name,
            token: None,
        }
    }

    pub fn get_type_name(&self) -> &TypeName {
        &self.name
    }

    pub fn mutate_type_name(&mut self) -> &mut TypeName {
        &mut self.name
    }

    pub fn get_namespace(&self) -> &Identifier {
        &self.namespace
    }

    pub fn mutate_namespace(&mut self) -> &mut Identifier {
        &mut self.namespace
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

    pub fn clear_comments(&mut self) {
        self.namespace.clear_comments();
        self.name.clear_comments();
        if let Some(token) = &mut self.token {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.namespace.clear_whitespaces();
        self.name.clear_whitespaces();
        if let Some(token) = &mut self.token {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.namespace.replace_referenced_tokens(code);
        self.name.replace_referenced_tokens(code);
        if let Some(token) = &mut self.token {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.namespace.shift_token_line(amount);
        self.name.shift_token_line(amount);
        if let Some(token) = &mut self.token {
            token.shift_token_line(amount);
        }
    }
}
