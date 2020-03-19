use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::Expression;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Length,
    Minus,
    Not,
}

impl ToLua for UnaryOperator {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::Length => generator.push_char('#'),
            Self::Minus => generator.push_char('-'),
            Self::Not => generator.push_str("not"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryExpression {
    operator: UnaryOperator,
    expression: Expression,
}

impl UnaryExpression {
    pub fn new(operator: UnaryOperator, expression: Expression) -> Self {
        Self {
            operator,
            expression,
        }
    }

    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }

    pub fn mutate_expression(&mut self) -> &mut Expression {
        &mut self.expression
    }

    pub fn operator(&self) -> UnaryOperator {
        self.operator
    }
}

impl ToLua for UnaryExpression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        self.operator.to_lua(generator);
        self.expression.to_lua(generator);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_unary_expression() {
        let output = UnaryExpression::new(
            UnaryOperator::Not,
            Expression::True,
        ).to_lua_string();

        assert_eq!(output, "not true");
    }
}
