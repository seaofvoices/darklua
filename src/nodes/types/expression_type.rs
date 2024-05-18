use crate::nodes::{Expression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionType {
    expression: Box<Expression>,
    tokens: Option<ExpressionTypeTokens>,
}

impl ExpressionType {
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: Box::new(expression.into()),
            tokens: None,
        }
    }

    #[inline]
    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }

    #[inline]
    pub fn mutate_expression(&mut self) -> &mut Expression {
        &mut self.expression
    }

    pub fn with_tokens(mut self, tokens: ExpressionTypeTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: ExpressionTypeTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&ExpressionTypeTokens> {
        self.tokens.as_ref()
    }

    super::impl_token_fns!(iter = [tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionTypeTokens {
    pub r#typeof: Token,
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
}

impl ExpressionTypeTokens {
    super::impl_token_fns!(target = [r#typeof, opening_parenthese, closing_parenthese]);
}
