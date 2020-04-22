use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::{Block, Expression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfBranch {
    condition: Expression,
    block: Block,
}

impl IfBranch {
    pub fn new(condition: Expression, block: Block) -> Self {
        Self {
            condition,
            block,
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
        let mut blocks: Vec<&mut Block> = self.branches.iter_mut()
            .map(|branch| branch.mutate_block())
            .collect();

        if let Some(else_block) = &mut self.else_block {
            blocks.push(else_block);
        };

        blocks
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
    pub fn else_block(&self) -> &Option<Block> {
        &self.else_block
    }

    #[inline]
    pub fn mutate_else_block(&mut self) -> &mut Option<Block> {
        &mut self.else_block
    }

    #[inline]
    pub fn take_else_block(&mut self) -> Option<Block> {
        self.else_block.take()
    }
}

impl ToLua for IfStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        self.branches.iter().enumerate()
            .for_each(|(index, branch)| {
                if index == 0 {
                    generator.push_str("if");
                } else {
                    generator.push_str("elseif");
                }
                branch.get_condition().to_lua(generator);
                generator.push_str("then");
                branch.get_block().to_lua(generator);
            });

        if let Some(else_block) = &self.else_block {
            generator.push_str("else");
            else_block.to_lua(generator);
        };

        generator.push_str("end");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_empty_if() {
        let output = IfStatement::create(Expression::False, Block::default())
            .to_lua_string();

        assert_eq!(output, "if false then end");
    }

    #[test]
    fn generate_empty_if_with_else_block() {
        let output = IfStatement::create(Expression::False, Block::default())
            .with_else_block(Block::default())
            .to_lua_string();

        assert_eq!(output, "if false then else end");
    }

    #[test]
    fn generate_empty_if_with_multiple_branch() {
        let output = IfStatement::create(Expression::False, Block::default())
            .with_branch(Expression::False, Block::default())
            .to_lua_string();

        assert_eq!(output, "if false then elseif false then end");
    }
}
