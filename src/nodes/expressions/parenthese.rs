use crate::nodes::{Expression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentheseTokens {
    pub left_parenthese: Token,
    pub right_parenthese: Token,
}

impl ParentheseTokens {
    pub fn clear_comments(&mut self) {
        self.left_parenthese.clear_comments();
        self.right_parenthese.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.left_parenthese.clear_whitespaces();
        self.right_parenthese.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.left_parenthese.replace_referenced_tokens(code);
        self.right_parenthese.replace_referenced_tokens(code);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParentheseExpression {
    expression: Expression,
    tokens: Option<ParentheseTokens>,
}

impl ParentheseExpression {
    pub fn new<E: Into<Expression>>(expression: E) -> Self {
        Self {
            expression: expression.into(),
            tokens: None,
        }
    }

    #[inline]
    pub fn inner_expression(&self) -> &Expression {
        &self.expression
    }

    #[inline]
    pub fn into_inner_expression(self) -> Expression {
        self.expression
    }

    #[inline]
    pub fn mutate_inner_expression(&mut self) -> &mut Expression {
        &mut self.expression
    }

    pub fn with_tokens(mut self, tokens: ParentheseTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: ParentheseTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&ParentheseTokens> {
        self.tokens.as_ref()
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
}
