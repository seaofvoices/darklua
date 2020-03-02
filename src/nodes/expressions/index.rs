use crate::nodes::{
    Expression,
    Prefix,
};
use crate::lua_generator::{LuaGenerator, ToLua};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexExpression {
    prefix: Prefix,
    index: Expression,
}

impl IndexExpression {
    pub fn new<E: Into<Expression>>(prefix: Prefix, expression: E) -> Self {
        Self {
            prefix,
            index: expression.into(),
        }
    }

    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }

    pub fn mutate_index(&mut self) -> &mut Expression {
        &mut self.index
    }
}

impl ToLua for IndexExpression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        self.prefix.to_lua(generator);
        generator.push_char('[');
        self.index.to_lua(generator);
        generator.push_char(']');
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_identifier() {
        let output = IndexExpression::new(
            Prefix::from_name("foo"),
            Prefix::from_name("bar"),
        ).to_lua_string();

        assert_eq!(output, "foo[bar]");
    }
}
