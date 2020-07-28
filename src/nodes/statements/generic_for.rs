use crate::nodes::{
    Block,
    Expression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericForStatement {
    identifiers: Vec<String>,
    expressions: Vec<Expression>,
    block: Block,
}

impl GenericForStatement {
    pub fn new(identifiers: Vec<String>, expressions: Vec<Expression>, block: Block) -> Self {
        Self {
            identifiers,
            expressions,
            block,
        }
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_identifiers(&self) -> &Vec<String> {
        &self.identifiers
    }

    #[inline]
    pub fn get_expressions(&self) -> &Vec<Expression> {
        &self.expressions
    }

    #[inline]
    pub fn mutate_identifiers(&mut self) -> &mut Vec<String> {
        &mut self.identifiers
    }

    #[inline]
    pub fn mutate_expressions(&mut self) -> &mut Vec<Expression> {
        &mut self.expressions
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }
}
