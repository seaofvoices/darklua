use crate::nodes::{
    Block,
    Expression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhileStatement {
    block: Block,
    condition: Expression,
}

impl WhileStatement {
    pub fn new(block: Block, condition: Expression) -> Self {
        Self {
            block,
            condition,
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
}
