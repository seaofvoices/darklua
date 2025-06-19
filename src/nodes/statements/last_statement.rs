use crate::nodes::{Expression, Token};

/// Tokens associated with a return statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnTokens {
    pub r#return: Token,
    /// The tokens for the commas between expressions.
    pub commas: Vec<Token>,
}

impl ReturnTokens {
    super::impl_token_fns!(
        target = [r#return]
        iter = [commas]
    );
}

/// Represents a return statement.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReturnStatement {
    expressions: Vec<Expression>,
    tokens: Option<ReturnTokens>,
}

impl ReturnStatement {
    /// Creates a new return statement with the given expressions.
    pub fn new(expressions: Vec<Expression>) -> Self {
        Self {
            expressions,
            tokens: None,
        }
    }

    /// Creates a new ReturnStatement with one expression.
    /// ```rust
    /// # use darklua_core::nodes::{Expression, ReturnStatement};
    ///
    /// let statement = ReturnStatement::one(true);
    ///
    /// // unknown case
    /// assert_eq!(statement.len(), 1);
    /// ```
    pub fn one<E: Into<Expression>>(expression: E) -> Self {
        Self {
            expressions: vec![expression.into()],
            tokens: None,
        }
    }

    /// Adds an expression to this return statement.
    pub fn with_expression<E: Into<Expression>>(mut self, expression: E) -> Self {
        self.expressions.push(expression.into());
        self
    }

    /// Returns an iterator over the expressions.
    #[inline]
    pub fn iter_expressions(&self) -> impl Iterator<Item = &Expression> {
        self.expressions.iter()
    }

    /// Converts this return statement into an iterator over its expressions.
    #[inline]
    pub fn into_iter_expressions(self) -> impl Iterator<Item = Expression> {
        self.expressions.into_iter()
    }

    /// Returns a mutable iterator over the expressions.
    #[inline]
    pub fn iter_mut_expressions(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.expressions.iter_mut()
    }

    /// Returns whether this return statement has no expressions.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.expressions.is_empty()
    }

    /// Returns the number of expressions.
    #[inline]
    pub fn len(&self) -> usize {
        self.expressions.len()
    }

    /// Sets the tokens for this return statement.
    pub fn with_tokens(mut self, tokens: ReturnTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Sets the tokens for this return statement.
    #[inline]
    pub fn set_tokens(&mut self, tokens: ReturnTokens) {
        self.tokens = Some(tokens);
    }

    /// Returns the tokens for this return statement, if any.
    #[inline]
    pub fn get_tokens(&self) -> Option<&ReturnTokens> {
        self.tokens.as_ref()
    }

    /// Returns a mutable reference to the tokens, if any.
    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut ReturnTokens> {
        self.tokens.as_mut()
    }

    super::impl_token_fns!(iter = [tokens]);
}

/// Represents a statement that can appear as the last statement in a block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LastStatement {
    Break(Option<Token>),
    Continue(Option<Token>),
    Return(ReturnStatement),
}

impl LastStatement {
    /// Creates a new break statement without a token.
    #[inline]
    pub fn new_break() -> Self {
        Self::Break(None)
    }

    /// Creates a new continue statement without a token.
    #[inline]
    pub fn new_continue() -> Self {
        Self::Continue(None)
    }
}

impl From<ReturnStatement> for LastStatement {
    fn from(statement: ReturnStatement) -> Self {
        Self::Return(statement)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_return_statement_is_empty() {
        assert!(ReturnStatement::default().is_empty())
    }
}
