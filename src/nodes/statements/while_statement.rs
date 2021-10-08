use crate::nodes::{token::Token, Block, Expression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhileTokens {
    pub r#while: Token,
    pub r#do: Token,
    pub end: Token,
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
}
