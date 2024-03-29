use crate::nodes::{Expression, Token, Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCastExpression {
    expression: Box<Expression>,
    r#type: Type,
    token: Option<Token>,
}

impl TypeCastExpression {
    pub fn new(expression: impl Into<Expression>, r#type: impl Into<Type>) -> Self {
        Self {
            expression: Box::new(expression.into()),
            r#type: r#type.into(),
            token: None,
        }
    }

    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }

    pub fn mutate_expression(&mut self) -> &mut Expression {
        &mut self.expression
    }

    pub fn get_type(&self) -> &Type {
        &self.r#type
    }

    pub fn mutate_type(&mut self) -> &mut Type {
        &mut self.r#type
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

    pub fn needs_parentheses(expression: &Expression) -> bool {
        matches!(
            expression,
            Expression::Binary(_)
                | Expression::Unary(_)
                | Expression::TypeCast(_)
                | Expression::If(_)
        )
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
}

#[cfg(test)]
mod test {
    use crate::nodes::IfExpression;

    use super::*;

    #[test]
    fn if_expression_needs_parentheses() {
        assert!(TypeCastExpression::needs_parentheses(&Expression::from(
            IfExpression::new(true, false, true)
        )));
    }
}
