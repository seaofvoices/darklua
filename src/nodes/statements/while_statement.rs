use crate::nodes::{token::Token, Block, Expression};

/// Tokens associated with a while statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhileTokens {
    pub r#while: Token,
    pub r#do: Token,
    pub end: Token,
}

impl WhileTokens {
    super::impl_token_fns!(target = [r#while, r#do, end]);
}

/// Represents a while loop statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhileStatement {
    block: Block,
    condition: Expression,
    tokens: Option<WhileTokens>,
}

impl WhileStatement {
    /// Creates a new while statement with the given block and condition.
    pub fn new<B: Into<Block>, E: Into<Expression>>(block: B, condition: E) -> Self {
        Self {
            block: block.into(),
            condition: condition.into(),
            tokens: None,
        }
    }

    /// Sets the tokens for this while statement.
    pub fn with_tokens(mut self, tokens: WhileTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Returns the loop's block.
    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    /// Returns the condition for this while loop.
    #[inline]
    pub fn get_condition(&self) -> &Expression {
        &self.condition
    }

    /// Returns a mutable reference to the block.
    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    /// Returns a mutable reference to the condition.
    #[inline]
    pub fn mutate_condition(&mut self) -> &mut Expression {
        &mut self.condition
    }

    /// Sets the tokens for this while statement.
    #[inline]
    pub fn set_tokens(&mut self, tokens: WhileTokens) {
        self.tokens = Some(tokens);
    }

    /// Returns the tokens for this while statement, if any.
    #[inline]
    pub fn get_tokens(&self) -> Option<&WhileTokens> {
        self.tokens.as_ref()
    }

    /// Returns a mutable reference to the tokens, if any.
    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut WhileTokens> {
        self.tokens.as_mut()
    }

    super::impl_token_fns!(iter = [tokens]);
}
