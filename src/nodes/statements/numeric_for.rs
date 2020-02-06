use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::{
    Block,
    Expression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericForStatement {
    identifier: String,
    start: Expression,
    end: Expression,
    step: Option<Expression>,
    block: Block,
}

impl NumericForStatement {
    pub fn new(
        identifier: String,
        start: Expression,
        end: Expression,
        step: Option<Expression>,
        block: Block
    ) -> Self {
        Self {
            identifier,
            start,
            end,
            step,
            block,
        }
    }

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn mutate_start(&mut self) -> &mut Expression {
        &mut self.start
    }

    pub fn mutate_end(&mut self) -> &mut Expression {
        &mut self.end
    }

    pub fn mutate_step(&mut self) -> &mut Option<Expression> {
        &mut self.step
    }

    pub fn get_identifier(&self) -> &String {
        &self.identifier
    }

    pub fn set_identifier<S: Into<String>>(&mut self, identifier: S) {
        self.identifier = identifier.into();
    }
}

impl ToLua for NumericForStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("for");
        generator.push_str(&self.identifier);
        generator.push_char('=');
        self.start.to_lua(generator);
        generator.push_char(',');
        self.end.to_lua(generator);

        if let Some(step) = &self.step {
            generator.push_char(',');
            step.to_lua(generator);
        }

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
    fn generate_empty_numeric_for() {
        let output = NumericForStatement::new(
            "i".to_owned(),
            Expression::Identifier("start".to_owned()),
            Expression::Identifier("max".to_owned()),
            None,
            Block::default()
        ).to_lua_string();

        assert_eq!(output, "for i=start,max do end");
    }

    #[test]
    fn generate_empty_numeric_for_with_step() {
        let output = NumericForStatement::new(
            "i".to_owned(),
            Expression::Identifier("start".to_owned()),
            Expression::Identifier("max".to_owned()),
            Some(Expression::Identifier("step".to_owned())),
            Block::default()
        ).to_lua_string();

        assert_eq!(output, "for i=start,max,step do end");
    }
}
