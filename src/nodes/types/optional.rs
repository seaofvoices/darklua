use crate::nodes::Token;

use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionalType {
    inner_type: Box<Type>,
    token: Option<Token>,
}

impl OptionalType {
    pub fn new(r#type: impl Into<Type>) -> Self {
        Self {
            inner_type: Box::new(r#type.into()),
            token: None,
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

    pub fn needs_parentheses(r#type: &Type) -> bool {
        matches!(
            r#type,
            Type::Intersection(_) | Type::Union(_) | Type::Function(_)
        )
    }

    super::impl_token_fns!(iter = [token]);
}
