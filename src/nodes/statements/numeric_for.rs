use crate::nodes::{Block, Expression, Identifier, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericForTokens {
    pub r#for: Token,
    pub equal: Token,
    pub r#do: Token,
    pub end: Token,
    pub end_comma: Token,
    pub step_comma: Option<Token>,
}

impl NumericForTokens {
    pub fn clear_comments(&mut self) {
        self.r#for.clear_comments();
        self.equal.clear_comments();
        self.r#do.clear_comments();
        self.end.clear_comments();
        self.end_comma.clear_comments();
        if let Some(token) = &mut self.step_comma {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.r#for.clear_whitespaces();
        self.equal.clear_whitespaces();
        self.r#do.clear_whitespaces();
        self.end.clear_whitespaces();
        self.end_comma.clear_whitespaces();
        if let Some(token) = &mut self.step_comma {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.r#for.replace_referenced_tokens(code);
        self.equal.replace_referenced_tokens(code);
        self.r#do.replace_referenced_tokens(code);
        self.end.replace_referenced_tokens(code);
        self.end_comma.replace_referenced_tokens(code);
        if let Some(token) = &mut self.step_comma {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.r#for.shift_token_line(amount);
        self.equal.shift_token_line(amount);
        self.r#do.shift_token_line(amount);
        self.end.shift_token_line(amount);
        self.end_comma.shift_token_line(amount);
        if let Some(token) = &mut self.step_comma {
            token.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericForStatement {
    identifier: Identifier,
    start: Expression,
    end: Expression,
    step: Option<Expression>,
    block: Block,
    tokens: Option<NumericForTokens>,
}

impl NumericForStatement {
    pub fn new<S: Into<Identifier>, E1: Into<Expression>, E2: Into<Expression>, B: Into<Block>>(
        identifier: S,
        start: E1,
        end: E2,
        step: Option<Expression>,
        block: B,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            start: start.into(),
            end: end.into(),
            step,
            block: block.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: NumericForTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: NumericForTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&NumericForTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn get_start(&self) -> &Expression {
        &self.start
    }

    #[inline]
    pub fn mutate_start(&mut self) -> &mut Expression {
        &mut self.start
    }

    #[inline]
    pub fn get_end(&self) -> &Expression {
        &self.end
    }

    #[inline]
    pub fn mutate_end(&mut self) -> &mut Expression {
        &mut self.end
    }

    #[inline]
    pub fn get_step(&self) -> Option<&Expression> {
        self.step.as_ref()
    }

    #[inline]
    pub fn mutate_step(&mut self) -> &mut Option<Expression> {
        &mut self.step
    }

    #[inline]
    pub fn get_identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut Identifier {
        &mut self.identifier
    }

    #[inline]
    pub fn set_identifier<S: Into<Identifier>>(&mut self, identifier: S) {
        self.identifier = identifier.into();
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
