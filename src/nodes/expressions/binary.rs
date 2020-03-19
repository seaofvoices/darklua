use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::Expression;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    And,
    Or,
    Equal,
    NotEqual,
    LowerThan,
    LowerOrEqualThan,
    GreatherThan,
    GreatherOrEqualThan,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Caret,
    Concat,
}

impl ToLua for BinaryOperator {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::And => generator.push_str("and"),
            Self::Or => generator.push_str("or"),
            Self::Equal => generator.push_str("=="),
            Self::NotEqual => generator.push_str("~="),
            Self::LowerThan => generator.push_char('<'),
            Self::LowerOrEqualThan => generator.push_str("<="),
            Self::GreatherThan => generator.push_char('>'),
            Self::GreatherOrEqualThan => generator.push_str(">="),
            Self::Plus => generator.push_char('+'),
            Self::Minus => generator.push_char('-'),
            Self::Asterisk => generator.push_char('*'),
            Self::Slash => generator.push_char('/'),
            Self::Percent => generator.push_char('%'),
            Self::Caret => generator.push_char('^'),
            Self::Concat => generator.push_str(".."),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryExpression {
    operator: BinaryOperator,
    left: Expression,
    right: Expression,
}

impl BinaryExpression {
    pub fn new(operator: BinaryOperator, left: Expression, right: Expression) -> Self {
        Self {
            operator,
            left,
            right,
        }
    }

    pub fn mutate_left(&mut self) -> &mut Expression {
        &mut self.left
    }

    pub fn mutate_right(&mut self) -> &mut Expression {
        &mut self.right
    }

    pub fn left(&self) -> &Expression {
        &self.left
    }

    pub fn right(&self) -> &Expression {
        &self.right
    }

    pub fn operator(&self) -> BinaryOperator {
        self.operator
    }
}

impl ToLua for BinaryExpression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        self.left.to_lua(generator);
        self.operator.to_lua(generator);
        self.right.to_lua(generator);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod snapshot {
        use super::*;

        use insta::assert_snapshot;

        #[test]
        fn and_expression() {
            let expression = BinaryExpression::new(
                BinaryOperator::And,
                Expression::True,
                Expression::False
            );

            assert_snapshot!("and_expression", expression.to_lua_string());
        }

        #[test]
        fn equal_expression() {
            let expression = BinaryExpression::new(
                BinaryOperator::Equal,
                Expression::True,
                Expression::False
            );

            assert_snapshot!("equal_expression", expression.to_lua_string());
        }
    }
}
