use crate::nodes::{Block, Expression, Token, TypedIdentifier};

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
    super::impl_token_fns!(
        target = [r#for, equal, r#do, end, end_comma]
        iter = [step_comma]
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericForStatement {
    identifier: TypedIdentifier,
    start: Expression,
    end: Expression,
    step: Option<Expression>,
    block: Block,
    tokens: Option<NumericForTokens>,
}

impl NumericForStatement {
    pub fn new<
        S: Into<TypedIdentifier>,
        E1: Into<Expression>,
        E2: Into<Expression>,
        B: Into<Block>,
    >(
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
    pub fn mutate_tokens(&mut self) -> Option<&mut NumericForTokens> {
        self.tokens.as_mut()
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
    pub fn get_identifier(&self) -> &TypedIdentifier {
        &self.identifier
    }

    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut TypedIdentifier {
        &mut self.identifier
    }

    #[inline]
    pub fn set_identifier<S: Into<TypedIdentifier>>(&mut self, identifier: S) {
        self.identifier = identifier.into();
    }

    pub fn clear_types(&mut self) {
        self.identifier.remove_type();
    }

    super::impl_token_fns!(target = [identifier] iter = [tokens]);
}
