use crate::nodes::{Block, Expression, Token, TypedIdentifier};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericForTokens {
    pub r#for: Token,
    pub r#in: Token,
    pub r#do: Token,
    pub end: Token,
    pub identifier_commas: Vec<Token>,
    pub value_commas: Vec<Token>,
}

impl GenericForTokens {
    super::impl_token_fns!(
        target = [r#for, r#in, r#do, end]
        iter = [identifier_commas, value_commas]
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericForStatement {
    identifiers: Vec<TypedIdentifier>,
    expressions: Vec<Expression>,
    block: Block,
    tokens: Option<GenericForTokens>,
}

impl GenericForStatement {
    pub fn new<B: Into<Block>>(
        identifiers: Vec<TypedIdentifier>,
        expressions: Vec<Expression>,
        block: B,
    ) -> Self {
        Self {
            identifiers,
            expressions,
            block: block.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: GenericForTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: GenericForTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&GenericForTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut GenericForTokens> {
        self.tokens.as_mut()
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_identifiers(&self) -> &Vec<TypedIdentifier> {
        &self.identifiers
    }

    #[inline]
    pub fn iter_identifiers(&self) -> impl Iterator<Item = &TypedIdentifier> {
        self.identifiers.iter()
    }

    #[inline]
    pub fn get_expressions(&self) -> &Vec<Expression> {
        &self.expressions
    }

    #[inline]
    pub fn iter_expressions(&self) -> impl Iterator<Item = &Expression> {
        self.expressions.iter()
    }

    #[inline]
    pub fn iter_mut_identifiers(&mut self) -> impl Iterator<Item = &mut TypedIdentifier> {
        self.identifiers.iter_mut()
    }

    #[inline]
    pub fn iter_mut_expressions(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.expressions.iter_mut()
    }

    #[inline]
    pub fn mutate_expressions(&mut self) -> &mut Vec<Expression> {
        &mut self.expressions
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn identifiers_len(&self) -> usize {
        self.identifiers.len()
    }

    #[inline]
    pub fn expressions_len(&self) -> usize {
        self.expressions.len()
    }

    pub fn clear_types(&mut self) {
        for identifier in &mut self.identifiers {
            identifier.remove_type();
        }
    }

    super::impl_token_fns!(iter = [tokens, identifiers]);
}
