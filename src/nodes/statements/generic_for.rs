use crate::nodes::{Block, Expression, Identifier, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericForTokens {
    pub r#for: Token,
    pub r#in: Token,
    pub r#do: Token,
    pub end: Token,
    pub identifier_commas: Vec<Token>,
    pub value_commas: Vec<Token>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericForStatement {
    identifiers: Vec<Identifier>,
    expressions: Vec<Expression>,
    block: Block,
    tokens: Option<GenericForTokens>,
}

impl GenericForStatement {
    pub fn new<B: Into<Block>>(
        identifiers: Vec<Identifier>,
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
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_identifiers(&self) -> &Vec<Identifier> {
        &self.identifiers
    }

    #[inline]
    pub fn iter_identifiers(&self) -> impl Iterator<Item = &Identifier> {
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
    pub fn iter_mut_identifiers(&mut self) -> impl Iterator<Item = &mut Identifier> {
        self.identifiers.iter_mut()
    }

    #[inline]
    pub fn iter_mut_expressions(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.expressions.iter_mut()
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
}
