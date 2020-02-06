use crate::lua_generator::{LuaGenerator, ToLua};
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

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn get_identifiers(&self) -> &Vec<String> {
        &self.identifiers
    }

    pub fn mutate_identifiers(&mut self) -> &mut Vec<String> {
        &mut self.identifiers
    }

    pub fn mutate_expressions(&mut self) -> &mut Vec<Expression> {
        &mut self.expressions
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }
}

impl ToLua for GenericForStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("for");

        generator.for_each_and_between(
            &self.identifiers,
            |generator, identifier| generator.push_str(identifier),
            |generator| generator.push_char(',')
        );

        generator.push_str("in");

        generator.for_each_and_between(
            &self.expressions,
            |generator, expression| expression.to_lua(generator),
            |generator| generator.push_char(',')
        );

        generator.push_str("do");
        self.block.to_lua(generator);
        generator.push_str("end");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nodes::Expression;

    #[test]
    fn generate_empty_generic_for() {
        let output = GenericForStatement::new(
            vec!["var".to_owned()],
            vec![Expression::True],
            Block::default()
        ).to_lua_string();

        assert_eq!(output, "for var in true do end");
    }
}
