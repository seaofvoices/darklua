mod binary;
mod field;
mod function;
mod index;
mod number;
mod prefix;
mod string;
mod table;
mod unary;

pub use binary::*;
pub use field::*;
pub use function::*;
pub use index::*;
pub use number::*;
pub use prefix::*;
pub use string::*;
pub use table::*;
pub use unary::*;

use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::FunctionCall;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Binary(Box<BinaryExpression>),
    Call(Box<FunctionCall>),
    False,
    Field(Box<FieldExpression>),
    Function(FunctionExpression),
    Identifier(String),
    Index(Box<IndexExpression>),
    Nil,
    Number(NumberExpression),
    Parenthese(Box<Expression>),
    String(StringExpression),
    Table(TableExpression),
    True,
    Unary(Box<UnaryExpression>),
    VariableArguments,
}

impl From<BinaryExpression> for Expression {
    fn from(binary: BinaryExpression) -> Expression {
        Expression::Binary(Box::new(binary))
    }
}

impl From<FunctionCall> for Expression {
    fn from(call: FunctionCall) -> Expression {
        Expression::Call(Box::new(call))
    }
}

impl From<FieldExpression> for Expression {
    fn from(field: FieldExpression) -> Expression {
        Expression::Field(Box::new(field))
    }
}

impl From<FunctionExpression> for Expression {
    fn from(function: FunctionExpression) -> Self {
        Expression::Function(function)
    }
}

impl From<IndexExpression> for Expression {
    fn from(index: IndexExpression) -> Self {
        Self::Index(Box::new(index))
    }
}

impl From<NumberExpression> for Expression {
    fn from(number: NumberExpression) -> Self {
        Self::Number(number)
    }
}

impl From<Prefix> for Expression {
    fn from(prefix: Prefix) -> Self {
        match prefix {
            Prefix::Call(call) => Self::Call(Box::new(call)),
            Prefix::Field(field) => Self::Field(field),
            Prefix::Identifier(name) => Self::Identifier(name),
            Prefix::Index(index) => Self::Index(index),
            Prefix::Parenthese(expression) => Self::Parenthese(Box::new(expression)),
        }
    }
}

impl From<StringExpression> for Expression {
    fn from(string: StringExpression) -> Self {
        Self::String(string)
    }
}

impl From<TableExpression> for Expression {
    fn from(table: TableExpression) -> Self {
        Self::Table(table)
    }
}

impl From<UnaryExpression> for Expression {
    fn from(unary: UnaryExpression) -> Self {
        Self::Unary(Box::new(unary))
    }
}

fn break_variable_arguments(last_string: &str) -> bool {
    if let Some('.') = last_string.chars().last() {
        true
    } else if let Some(first_char) = last_string.chars().next() {
        first_char == '.' || first_char.is_digit(10)
    } else {
        false
    }
}

impl ToLua for Expression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        match self {
            Self::Binary(binary_expression) => binary_expression.to_lua(generator),
            Self::Call(call) => call.to_lua(generator),
            Self::False => generator.push_str("false"),
            Self::Field(field) => field.to_lua(generator),
            Self::Function(function) => function.to_lua(generator),
            Self::Identifier(identifier) => generator.push_str(identifier),
            Self::Index(index) => index.to_lua(generator),
            Self::Nil => generator.push_str("nil"),
            Self::Number(number) => number.to_lua(generator),
            Self::Parenthese(expression) => {
                generator.push_char('(');
                expression.to_lua(generator);
                generator.push_char(')');
            }
            Self::String(string) => string.to_lua(generator),
            Self::Table(table) => table.to_lua(generator),
            Self::True => generator.push_str("true"),
            Self::Unary(unary_expression) => unary_expression.to_lua(generator),
            Self::VariableArguments => {
                generator.push_str_and_break_if("...", break_variable_arguments);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod to_lua {
        use super::*;

        #[test]
        fn generate_false_expression() {
            let output = Expression::False.to_lua_string();

            assert_eq!(output, "false");
        }

        #[test]
        fn generate_nil_expression() {
            let output = Expression::Nil.to_lua_string();

            assert_eq!(output, "nil");
        }

        #[test]
        fn generate_parenthese_expression() {
            let inner_expression = Box::new(Expression::True);
            let output = Expression::Parenthese(inner_expression).to_lua_string();

            assert_eq!(output, "(true)");
        }

        #[test]
        fn generate_true_expression() {
            let output = Expression::True.to_lua_string();

            assert_eq!(output, "true");
        }

        #[test]
        fn generate_variable_arguments_expression() {
            let output = Expression::VariableArguments.to_lua_string();

            assert_eq!(output, "...");
        }
    }
}
