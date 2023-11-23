use crate::nodes::{Expression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnTokens {
    pub r#return: Token,
    pub commas: Vec<Token>,
}

impl ReturnTokens {
    pub fn clear_comments(&mut self) {
        self.r#return.clear_comments();
        self.commas.iter_mut().for_each(Token::clear_comments);
    }

    pub fn clear_whitespaces(&mut self) {
        self.r#return.clear_whitespaces();
        self.commas.iter_mut().for_each(Token::clear_whitespaces);
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.r#return.replace_referenced_tokens(code);
        for comma in self.commas.iter_mut() {
            comma.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.r#return.shift_token_line(amount);
        for comma in self.commas.iter_mut() {
            comma.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReturnStatement {
    expressions: Vec<Expression>,
    tokens: Option<ReturnTokens>,
}

impl ReturnStatement {
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
    /// let statement = ReturnStatement::one(Expression::from(true));
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

    pub fn with_expression<E: Into<Expression>>(mut self, expression: E) -> Self {
        self.expressions.push(expression.into());
        self
    }

    #[inline]
    pub fn iter_expressions(&self) -> impl Iterator<Item = &Expression> {
        self.expressions.iter()
    }

    #[inline]
    pub fn into_iter_expressions(self) -> impl Iterator<Item = Expression> {
        self.expressions.into_iter()
    }

    #[inline]
    pub fn iter_mut_expressions(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.expressions.iter_mut()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.expressions.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.expressions.len()
    }

    pub fn with_tokens(mut self, tokens: ReturnTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: ReturnTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&ReturnTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut ReturnTokens> {
        self.tokens.as_mut()
    }

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LastStatement {
    Break(Option<Token>),
    Continue(Option<Token>),
    Return(ReturnStatement),
}

impl LastStatement {
    #[inline]
    pub fn new_break() -> Self {
        Self::Break(None)
    }

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
