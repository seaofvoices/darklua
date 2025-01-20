use crate::nodes::{Expression, Prefix, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexExpressionTokens {
    pub opening_bracket: Token,
    pub closing_bracket: Token,
}

impl IndexExpressionTokens {
    super::impl_token_fns!(target = [opening_bracket, closing_bracket]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexExpression {
    prefix: Prefix,
    index: Expression,
    tokens: Option<IndexExpressionTokens>,
}

impl IndexExpression {
    pub fn new<P: Into<Prefix>, E: Into<Expression>>(prefix: P, expression: E) -> Self {
        Self {
            prefix: prefix.into(),
            index: expression.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: IndexExpressionTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: IndexExpressionTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&IndexExpressionTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    #[inline]
    pub fn get_index(&self) -> &Expression {
        &self.index
    }

    #[inline]
    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }

    #[inline]
    pub fn mutate_index(&mut self) -> &mut Expression {
        &mut self.index
    }

    super::impl_token_fns!(iter = [tokens]);
}
