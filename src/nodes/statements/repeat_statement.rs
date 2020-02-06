use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::{
    Block,
    Expression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatStatement {
    block: Block,
    condition: Expression,
}

impl RepeatStatement {
    pub fn new(block: Block, condition: Expression) -> Self {
        Self {
            block,
            condition,
        }
    }

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn mutate_condition(&mut self) -> &mut Expression {
        &mut self.condition
    }
}

impl ToLua for RepeatStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("repeat");
        self.block.to_lua(generator);
        generator.push_str("until");
        self.condition.to_lua(generator);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_empty_repeat_statement() {
        let output = RepeatStatement::new(
            Block::default(),
            Expression::False
        ).to_lua_string();

        assert_eq!(output, "repeat until false");
    }
}
