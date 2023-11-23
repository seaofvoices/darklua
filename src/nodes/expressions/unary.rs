use crate::nodes::{Expression, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Length,
    Minus,
    Not,
}

impl UnaryOperator {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Length => "#",
            Self::Minus => "-",
            Self::Not => "not",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryExpression {
    operator: UnaryOperator,
    expression: Expression,
    token: Option<Token>,
}

impl UnaryExpression {
    pub fn new<E: Into<Expression>>(operator: UnaryOperator, expression: E) -> Self {
        Self {
            operator,
            expression: expression.into(),
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
    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }

    #[inline]
    pub fn mutate_expression(&mut self) -> &mut Expression {
        &mut self.expression
    }

    #[inline]
    pub fn operator(&self) -> UnaryOperator {
        self.operator
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
