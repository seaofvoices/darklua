use crate::nodes::Token;

use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntersectionType {
    left_type: Box<Type>,
    right_type: Box<Type>,
    token: Option<Token>,
}

impl IntersectionType {
    pub fn new(left_type: impl Into<Type>, right_type: impl Into<Type>) -> Self {
        Self {
            left_type: Box::new(left_type.into()),
            right_type: Box::new(right_type.into()),
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
    pub fn mutate_left(&mut self) -> &mut Type {
        &mut self.left_type
    }

    #[inline]
    pub fn mutate_right(&mut self) -> &mut Type {
        &mut self.right_type
    }

    #[inline]
    pub fn get_left(&self) -> &Type {
        &self.left_type
    }

    #[inline]
    pub fn get_right(&self) -> &Type {
        &self.right_type
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

    pub fn left_needs_parentheses(r#type: &Type) -> bool {
        matches!(
            r#type,
            Type::Optional(_) | Type::Union(_) | Type::Function(_) | Type::Intersection(_)
        )
    }

    pub fn right_needs_parentheses(r#type: &Type) -> bool {
        matches!(r#type, Type::Optional(_) | Type::Union(_))
    }
}
