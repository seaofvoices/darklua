use crate::nodes::{Block, Expression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfBranch {
    condition: Expression,
    block: Block,
}

impl IfBranch {
    pub fn new(condition: Expression, block: Block) -> Self {
        Self { condition, block }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfStatement {
    branches: Vec<IfBranch>,
    else_block: Option<Block>,
}

impl IfStatement {
    pub fn new(branches: Vec<IfBranch>, else_block: Option<Block>) -> Self {
        Self {
            branches,
            else_block,
        }
    }

    pub fn create(condition: Expression, block: Block) -> Self {
        Self {
            branches: vec![IfBranch::new(condition, block)],
            else_block: None,
        }
    }

    pub fn with_branch(mut self, condition: Expression, block: Block) -> Self {
        self.branches.push(IfBranch::new(condition, block));
        self
    }

    pub fn with_else_block(mut self, block: Block) -> Self {
        self.else_block.replace(block);
        self
    }

    pub fn mutate_all_blocks(&mut self) -> Vec<&mut Block> {
        let mut blocks: Vec<&mut Block> = self
            .branches
            .iter_mut()
            .map(|branch| branch.mutate_block())
            .collect();

        if let Some(else_block) = &mut self.else_block {
            blocks.push(else_block);
        };

        blocks
    }

    #[inline]
    pub fn get_branches(&self) -> &Vec<IfBranch> {
        &self.branches
    }

    #[inline]
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    #[inline]
    pub fn mutate_branches(&mut self) -> &mut Vec<IfBranch> {
        &mut self.branches
    }

    #[inline]
    pub fn push_branch<E: Into<Expression>, B: Into<Block>>(&mut self, condition: E, block: B) {
        self.branches
            .push(IfBranch::new(condition.into(), block.into()));
    }

    #[inline]
    pub fn get_else_block(&self) -> Option<&Block> {
        self.else_block.as_ref()
    }

    #[inline]
    pub fn mutate_else_block(&mut self) -> &mut Option<Block> {
        &mut self.else_block
    }

    #[inline]
    pub fn set_else_block<B: Into<Block>>(&mut self, block: B) {
        self.else_block = Some(block.into());
    }

    #[inline]
    pub fn take_else_block(&mut self) -> Option<Block> {
        self.else_block.take()
    }
}
