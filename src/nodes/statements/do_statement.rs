use crate::lua_generator::{LuaGenerator, ToLua};
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

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }
}

impl ToLua for DoStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("do");
        self.block.to_lua(generator);
        generator.push_str("end");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_empty_do_statement() {
        let output = DoStatement::default().to_lua_string();

        assert_eq!(output, "do end");
    }

    #[test]
    fn generate_nested_do_statement() {
        let inner_block = Block::default().with_statement(DoStatement::default());

        let output = DoStatement::new(inner_block).to_lua_string();

        assert_eq!(output, "do do end end");
    }
}
