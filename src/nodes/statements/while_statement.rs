use crate::nodes::{token::Token, Block, Expression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhileTokens {
    pub r#while: Token,
    pub r#do: Token,
    pub end: Token,
}

impl WhileTokens {
    pub fn clear_comments(&mut self) {
        self.r#while.clear_comments();
        self.r#do.clear_comments();
        self.end.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.r#while.clear_whitespaces();
        self.r#do.clear_whitespaces();
        self.end.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.r#while.replace_referenced_tokens(code);
        self.r#do.replace_referenced_tokens(code);
        self.end.replace_referenced_tokens(code);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhileStatement {
    block: Block,
    condition: Expression,
    tokens: Option<WhileTokens>,
}

impl WhileStatement {
    pub fn new<B: Into<Block>, E: Into<Expression>>(block: B, condition: E) -> Self {
        Self {
            block: block.into(),
            condition: condition.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: WhileTokens) -> Self {
        self.tokens = Some(tokens);
        self
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

    #[inline]
    pub fn set_tokens(&mut self, tokens: WhileTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&WhileTokens> {
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
