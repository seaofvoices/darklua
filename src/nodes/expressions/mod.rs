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

use std::num::FpCategory;

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

impl From<bool> for Expression {
    fn from(boolean: bool) -> Expression {
        if boolean { Expression::True } else { Expression::False }
    }
}

impl From<f64> for Expression {
    fn from(value: f64) -> Expression {
        match value.classify() {
            FpCategory::Nan => {
                BinaryExpression::new(
                    BinaryOperator::Slash,
                    DecimalNumber::new(0.0).into(),
                    DecimalNumber::new(0.0).into(),
                ).into()
            }
            FpCategory::Infinite => {
                BinaryExpression::new(
                    BinaryOperator::Slash,
                    Expression::from(if value.is_sign_positive() { 1.0 } else { -1.0 }),
                    DecimalNumber::new(0.0).into(),
                ).into()
            }
            FpCategory::Zero => {
                DecimalNumber::new(0.0).into()
            }
            FpCategory::Subnormal | FpCategory::Normal => {
                if value < 0.0 {
                    UnaryExpression::new(
                        UnaryOperator::Minus,
                        Expression::from(value.abs()),
                    ).into()
                } else {
                    if value < 0.1 {
                        let exponent = value.log10().floor();
                        let new_value = value / 10_f64.powf(exponent);

                        DecimalNumber::new(new_value)
                            .with_exponent(exponent as i64, true)
                            .into()
                    } else if value > 999.0 && (value / 100.0).fract() == 0.0 {
                        let mut exponent = value.log10().floor();
                        let mut power = 10_f64.powf(exponent);

                        while exponent > 2.0 && (value / power).fract() != 0.0 {
                            exponent -= 1.0;
                            power /= 10.0;
                        }

                        DecimalNumber::new(value / power)
                            .with_exponent(exponent as i64, true)
                            .into()
                    } else {
                        DecimalNumber::new(value).into()
                    }
                }
            }
        }
    }
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

impl From<DecimalNumber> for Expression {
    fn from(number: DecimalNumber) -> Self {
        Self::Number(NumberExpression::Decimal(number))
    }
}

impl From<HexNumber> for Expression {
    fn from(number: HexNumber) -> Self {
        Self::Number(NumberExpression::Hex(number))
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

    mod numbers {
        use super::*;

        macro_rules! snapshots {
            ($($name:ident($input:expr)),+) => {
                $(
                    mod $name {
                        use super::*;
                        use insta::assert_snapshot;
                        use insta::assert_debug_snapshot;

                        #[test]
                        fn expression() {
                            assert_debug_snapshot!(
                                "expression",
                                Expression::from($input)
                            );
                        }

                        #[test]
                        fn lua() {
                            assert_snapshot!(
                                "lua_float",
                                Expression::from($input).to_lua_string()
                            );
                        }
                    }
                )+
            };
        }

        snapshots!(
            snaphshot_1(1.0),
            snaphshot_0_5(0.5),
            snaphshot_123(123.0),
            snaphshot_0_005(0.005),
            snaphshot_nan(0.0/0.0),
            snaphshot_positive_infinity(1.0/0.0),
            snaphshot_negative_infinity(-1.0/0.0),
            snaphshot_very_small(1.2345e-50),
            snapshot_thousand(1000.0),
            snaphshot_very_large(1.2345e50),
            snapshot_float_below_thousand(100.25),
            snapshot_float_above_thousand(2000.05)
        );
    }

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
