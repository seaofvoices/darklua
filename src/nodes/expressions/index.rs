use crate::nodes::{Expression, Prefix, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexExpressionTokens {
    pub opening_bracket: Token,
    pub closing_bracket: Token,
}

impl IndexExpressionTokens {
    pub fn clear_comments(&mut self) {
        self.opening_bracket.clear_comments();
        self.closing_bracket.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_bracket.clear_whitespaces();
        self.closing_bracket.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_bracket.replace_referenced_tokens(code);
        self.closing_bracket.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.opening_bracket.shift_token_line(amount);
        self.closing_bracket.shift_token_line(amount);
    }
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
