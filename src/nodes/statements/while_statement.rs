use crate::lua_generator::{LuaGenerator, ToLua};
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

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn get_condition(&self) -> &Expression {
        &self.condition
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn mutate_condition(&mut self) -> &mut Expression {
        &mut self.condition
    }
}

impl ToLua for WhileStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("while");
        self.condition.to_lua(generator);
        generator.push_str("do");
        self.block.to_lua(generator);
        generator.push_str("end");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_empty_while_statement() {
        let output = WhileStatement::new(
            Block::default(),
            Expression::False
        ).to_lua_string();

        assert_eq!(output, "while false do end");
    }
}
