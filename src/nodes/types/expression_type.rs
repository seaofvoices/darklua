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
pub struct ExpressionTypeTokens {
    pub r#typeof: Token,
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
}

impl ExpressionTypeTokens {
    pub fn clear_comments(&mut self) {
        self.r#typeof.clear_comments();
        self.opening_parenthese.clear_comments();
        self.closing_parenthese.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.r#typeof.clear_whitespaces();
        self.opening_parenthese.clear_whitespaces();
        self.closing_parenthese.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.r#typeof.replace_referenced_tokens(code);
        self.opening_parenthese.replace_referenced_tokens(code);
        self.closing_parenthese.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.r#typeof.shift_token_line(amount);
        self.opening_parenthese.shift_token_line(amount);
        self.closing_parenthese.shift_token_line(amount);
    }
}
