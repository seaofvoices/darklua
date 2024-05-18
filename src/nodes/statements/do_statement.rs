use crate::nodes::{Block, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoTokens {
    pub r#do: Token,
    pub end: Token,
}

impl DoTokens {
    super::impl_token_fns!(target = [r#do, end]);
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

    super::impl_token_fns!(iter = [tokens]);
}
