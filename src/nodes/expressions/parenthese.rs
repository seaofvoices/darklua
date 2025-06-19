use crate::nodes::{Expression, Token};

/// Contains token information for a parenthesized expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentheseTokens {
    /// The left (opening) parenthesis token
    pub left_parenthese: Token,
    /// The right (closing) parenthesis token
    pub right_parenthese: Token,
}

impl ParentheseTokens {
    super::impl_token_fns!(target = [left_parenthese, right_parenthese]);
}

/// Represents a parenthesized expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentheseExpression {
    expression: Expression,
    tokens: Option<ParentheseTokens>,
}

impl ParentheseExpression {
    /// Creates a new parenthesized expression containing the given expression.
    pub fn new<E: Into<Expression>>(expression: E) -> Self {
        Self {
            expression: expression.into(),
            tokens: None,
        }
    }

    /// Returns a reference to the inner expression contained in the parentheses.
    #[inline]
    pub fn inner_expression(&self) -> &Expression {
        &self.expression
    }

    /// Consumes this parenthesized expression and returns the inner expression.
    #[inline]
    pub fn into_inner_expression(self) -> Expression {
        self.expression
    }

    /// Returns a mutable reference to the inner expression contained in the parentheses.
    #[inline]
    pub fn mutate_inner_expression(&mut self) -> &mut Expression {
        &mut self.expression
    }

    /// Attaches tokens to this parenthesized expression.
    pub fn with_tokens(mut self, tokens: ParentheseTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Attaches tokens to this parenthesized expression.
    #[inline]
    pub fn set_tokens(&mut self, tokens: ParentheseTokens) {
        self.tokens = Some(tokens);
    }

    /// Returns a reference to the tokens attached to this parenthesized expression, if any.
    #[inline]
    pub fn get_tokens(&self) -> Option<&ParentheseTokens> {
        self.tokens.as_ref()
    }

    /// Returns a mutable reference to the tokens attached to this parenthesized expression, if any.
    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut ParentheseTokens> {
        self.tokens.as_mut()
    }

    super::impl_token_fns!(iter = [tokens]);
}
