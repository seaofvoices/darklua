use crate::nodes::{Block, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoTokens {
    pub r#do: Token,
    pub end: Token,
}

impl DoTokens {
    pub fn clear_comments(&mut self) {
        self.r#do.clear_comments();
        self.end.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.r#do.clear_whitespaces();
        self.end.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.r#do.replace_referenced_tokens(code);
        self.end.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.r#do.shift_token_line(amount);
        self.end.shift_token_line(amount);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DoStatement {
    block: Block,
    tokens: Option<DoTokens>,
}

impl DoStatement {
    pub fn new(block: Block) -> Self {
        Self {
            block,
            tokens: None,
        }
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn with_tokens(mut self, tokens: DoTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: DoTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&DoTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut DoTokens> {
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
