use crate::nodes::Block;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DoStatement {
    block: Block,
}

impl DoStatement {
    pub fn new(block: Block) -> Self {
        Self {
            block,
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

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.block.is_empty()
    }
}
