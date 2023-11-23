use crate::nodes::{Block, Expression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatTokens {
    pub repeat: Token,
    pub until: Token,
}

impl RepeatTokens {
    pub fn clear_comments(&mut self) {
        self.repeat.clear_comments();
        self.until.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.repeat.clear_whitespaces();
        self.until.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.repeat.replace_referenced_tokens(code);
        self.until.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.repeat.shift_token_line(amount);
        self.until.shift_token_line(amount);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatStatement {
    block: Block,
    condition: Expression,
    tokens: Option<RepeatTokens>,
}

impl RepeatStatement {
    pub fn new<B: Into<Block>, E: Into<Expression>>(block: B, condition: E) -> Self {
        Self {
            block: block.into(),
            condition: condition.into(),
            tokens: None,
        }
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_condition(&self) -> &Expression {
        &self.condition
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn mutate_condition(&mut self) -> &mut Expression {
        &mut self.condition
    }

    pub fn with_tokens(mut self, tokens: RepeatTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: RepeatTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&RepeatTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut RepeatTokens> {
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
