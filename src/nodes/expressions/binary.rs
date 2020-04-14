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
    GreaterThan,
    GreaterOrEqualThan,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Caret,
    Concat,
}

impl BinaryOperator {
    #[inline]
    pub fn precedes(&self, other: Self) -> bool {
        self.get_precedence() > other.get_precedence()
    }

    #[inline]
    pub fn preceeds_unary_expression(&self) -> bool {
        match self {
            Self::Caret => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_left_associative(&self) -> bool {
        match self {
            Self::Caret | Self::Concat => false,
            _ => true,
        }
    }

    #[inline]
    pub fn is_right_associative(&self) -> bool {
        match self {
            Self::Caret | Self::Concat => true,
            _ => false,
        }
    }

    fn get_precedence(&self) -> u8 {
        match self {
            Self::Or => 0,
            Self::And => 1,
            Self::Equal | Self::NotEqual
            | Self::LowerThan | Self::LowerOrEqualThan
            | Self::GreaterThan | Self::GreaterOrEqualThan => 2,
            Self::Concat => 3,
            Self::Plus | Self::Minus => 4,
            Self::Asterisk | Self::Slash | Self::Percent => 5,
            Self::Caret => 7,
        }
    }
}

fn break_concat(last_string: &str) -> bool {
    if let Some('.') = last_string.chars().last() {
        true
    } else if let Some(first_char) = last_string.chars().next() {
        first_char == '.' || first_char.is_digit(10)
    } else {
        false
    }
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
            Self::GreaterThan => generator.push_char('>'),
            Self::GreaterOrEqualThan => generator.push_str(">="),
            Self::Plus => generator.push_char('+'),
            Self::Minus => generator.push_char('-'),
            Self::Asterisk => generator.push_char('*'),
            Self::Slash => generator.push_char('/'),
            Self::Percent => generator.push_char('%'),
            Self::Caret => generator.push_char('^'),
            Self::Concat => generator.push_str_and_break_if("..", break_concat),
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

    mod precedence {
        use super::*;

        use BinaryOperator::*;

        #[test]
        fn caret() {
            assert!(Caret.precedes(And));
            assert!(Caret.precedes(Or));
            assert!(Caret.precedes(Equal));
            assert!(Caret.precedes(NotEqual));
            assert!(Caret.precedes(LowerThan));
            assert!(Caret.precedes(LowerOrEqualThan));
            assert!(Caret.precedes(GreaterThan));
            assert!(Caret.precedes(GreaterOrEqualThan));
            assert!(Caret.precedes(Plus));
            assert!(Caret.precedes(Minus));
            assert!(Caret.precedes(Asterisk));
            assert!(Caret.precedes(Slash));
            assert!(Caret.precedes(Percent));
            assert!(Caret.precedes(Concat));
            assert!(!Caret.precedes(Caret));
            assert!(Caret.preceeds_unary_expression());
        }

        #[test]
        fn asterisk() {
            assert!(Asterisk.precedes(And));
            assert!(Asterisk.precedes(Or));
            assert!(Asterisk.precedes(Equal));
            assert!(Asterisk.precedes(NotEqual));
            assert!(Asterisk.precedes(LowerThan));
            assert!(Asterisk.precedes(LowerOrEqualThan));
            assert!(Asterisk.precedes(GreaterThan));
            assert!(Asterisk.precedes(GreaterOrEqualThan));
            assert!(Asterisk.precedes(Plus));
            assert!(Asterisk.precedes(Minus));
            assert!(!Asterisk.precedes(Asterisk));
            assert!(!Asterisk.precedes(Slash));
            assert!(!Asterisk.precedes(Percent));
            assert!(Asterisk.precedes(Concat));
            assert!(!Asterisk.precedes(Caret));
            assert!(!Asterisk.preceeds_unary_expression());
        }

        #[test]
        fn slash() {
            assert!(Slash.precedes(And));
            assert!(Slash.precedes(Or));
            assert!(Slash.precedes(Equal));
            assert!(Slash.precedes(NotEqual));
            assert!(Slash.precedes(LowerThan));
            assert!(Slash.precedes(LowerOrEqualThan));
            assert!(Slash.precedes(GreaterThan));
            assert!(Slash.precedes(GreaterOrEqualThan));
            assert!(Slash.precedes(Plus));
            assert!(Slash.precedes(Minus));
            assert!(!Slash.precedes(Asterisk));
            assert!(!Slash.precedes(Slash));
            assert!(!Slash.precedes(Percent));
            assert!(Slash.precedes(Concat));
            assert!(!Slash.precedes(Caret));
            assert!(!Slash.preceeds_unary_expression());
        }

        #[test]
        fn percent() {
            assert!(Percent.precedes(And));
            assert!(Percent.precedes(Or));
            assert!(Percent.precedes(Equal));
            assert!(Percent.precedes(NotEqual));
            assert!(Percent.precedes(LowerThan));
            assert!(Percent.precedes(LowerOrEqualThan));
            assert!(Percent.precedes(GreaterThan));
            assert!(Percent.precedes(GreaterOrEqualThan));
            assert!(Percent.precedes(Plus));
            assert!(Percent.precedes(Minus));
            assert!(!Percent.precedes(Asterisk));
            assert!(!Percent.precedes(Slash));
            assert!(!Percent.precedes(Percent));
            assert!(Percent.precedes(Concat));
            assert!(!Percent.precedes(Caret));
            assert!(!Percent.preceeds_unary_expression());
        }

        #[test]
        fn plus() {
            assert!(Plus.precedes(And));
            assert!(Plus.precedes(Or));
            assert!(Plus.precedes(Equal));
            assert!(Plus.precedes(NotEqual));
            assert!(Plus.precedes(LowerThan));
            assert!(Plus.precedes(LowerOrEqualThan));
            assert!(Plus.precedes(GreaterThan));
            assert!(Plus.precedes(GreaterOrEqualThan));
            assert!(!Plus.precedes(Plus));
            assert!(!Plus.precedes(Minus));
            assert!(!Plus.precedes(Asterisk));
            assert!(!Plus.precedes(Slash));
            assert!(!Plus.precedes(Percent));
            assert!(Plus.precedes(Concat));
            assert!(!Plus.precedes(Caret));
            assert!(!Plus.preceeds_unary_expression());
        }

        #[test]
        fn minus() {
            assert!(Minus.precedes(And));
            assert!(Minus.precedes(Or));
            assert!(Minus.precedes(Equal));
            assert!(Minus.precedes(NotEqual));
            assert!(Minus.precedes(LowerThan));
            assert!(Minus.precedes(LowerOrEqualThan));
            assert!(Minus.precedes(GreaterThan));
            assert!(Minus.precedes(GreaterOrEqualThan));
            assert!(!Minus.precedes(Plus));
            assert!(!Minus.precedes(Minus));
            assert!(!Minus.precedes(Asterisk));
            assert!(!Minus.precedes(Slash));
            assert!(!Minus.precedes(Percent));
            assert!(Minus.precedes(Concat));
            assert!(!Minus.precedes(Caret));
            assert!(!Minus.preceeds_unary_expression());
        }

        #[test]
        fn concat() {
            assert!(Concat.precedes(And));
            assert!(Concat.precedes(Or));
            assert!(Concat.precedes(Equal));
            assert!(Concat.precedes(NotEqual));
            assert!(Concat.precedes(LowerThan));
            assert!(Concat.precedes(LowerOrEqualThan));
            assert!(Concat.precedes(GreaterThan));
            assert!(Concat.precedes(GreaterOrEqualThan));
            assert!(!Concat.precedes(Plus));
            assert!(!Concat.precedes(Minus));
            assert!(!Concat.precedes(Asterisk));
            assert!(!Concat.precedes(Slash));
            assert!(!Concat.precedes(Percent));
            assert!(!Concat.precedes(Concat));
            assert!(!Concat.precedes(Caret));
            assert!(!Concat.preceeds_unary_expression());
        }

        #[test]
        fn and() {
            assert!(!And.precedes(And));
            assert!(And.precedes(Or));
            assert!(!And.precedes(Equal));
            assert!(!And.precedes(NotEqual));
            assert!(!And.precedes(LowerThan));
            assert!(!And.precedes(LowerOrEqualThan));
            assert!(!And.precedes(GreaterThan));
            assert!(!And.precedes(GreaterOrEqualThan));
            assert!(!And.precedes(Plus));
            assert!(!And.precedes(Minus));
            assert!(!And.precedes(Asterisk));
            assert!(!And.precedes(Slash));
            assert!(!And.precedes(Percent));
            assert!(!And.precedes(Concat));
            assert!(!And.precedes(Caret));
            assert!(!And.preceeds_unary_expression());
        }
    }

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
