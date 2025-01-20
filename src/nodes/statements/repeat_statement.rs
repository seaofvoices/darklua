use crate::nodes::{Block, Expression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatTokens {
    pub repeat: Token,
    pub until: Token,
}

impl RepeatTokens {
    super::impl_token_fns!(target = [repeat, until]);
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

    #[inline]
    pub(crate) fn mutate_block_and_condition(&mut self) -> (&mut Block, &mut Expression) {
        (&mut self.block, &mut self.condition)
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

    super::impl_token_fns!(iter = [tokens]);
}
