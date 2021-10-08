use crate::nodes::{Block, Expression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfBranchTokens {
    pub elseif: Token,
    pub then: Token,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfBranch {
    condition: Expression,
    block: Block,
    tokens: Option<IfBranchTokens>,
}

impl IfBranch {
    pub fn new<E: Into<Expression>, B: Into<Block>>(condition: E, block: B) -> Self {
        Self {
            condition: condition.into(),
            block: block.into(),
            tokens: None,
        }
    }

    pub fn empty<E: Into<Expression>>(condition: E) -> Self {
        Self {
            condition: condition.into(),
            block: Block::default(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: IfBranchTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: IfBranchTokens) {
        self.tokens = Some(tokens);
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
pub struct IfStatementTokens {
    pub r#if: Token,
    pub then: Token,
    pub end: Token,
    pub r#else: Option<Token>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfStatement {
    branches: Vec<IfBranch>,
    else_block: Option<Block>,
    tokens: Option<IfStatementTokens>,
}

impl IfStatement {
    pub fn new(branches: Vec<IfBranch>, else_block: Option<Block>) -> Self {
        Self {
            branches,
            else_block,
            tokens: None,
        }
    }

    pub fn create<E: Into<Expression>, B: Into<Block>>(condition: E, block: B) -> Self {
        Self {
            branches: vec![IfBranch::new(condition, block)],
            else_block: None,
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: IfStatementTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: IfStatementTokens) {
        self.tokens = Some(tokens);
    }

    pub fn with_branch(mut self, branch: IfBranch) -> Self {
        self.branches.push(branch);
        self
    }

    pub fn with_new_branch<E: Into<Expression>, B: Into<Block>>(
        mut self,
        condition: E,
        block: B,
    ) -> Self {
        self.branches.push(IfBranch::new(condition, block));
        self
    }

    pub fn with_else_block<B: Into<Block>>(mut self, block: B) -> Self {
        self.else_block.replace(block.into());
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
    pub fn push_new_branch<E: Into<Expression>, B: Into<Block>>(&mut self, condition: E, block: B) {
        self.branches
            .push(IfBranch::new(condition.into(), block.into()));
    }

    #[inline]
    pub fn push_branch(&mut self, branch: IfBranch) {
        self.branches.push(branch);
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
