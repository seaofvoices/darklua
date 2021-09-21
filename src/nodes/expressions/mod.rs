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

use crate::nodes::{FunctionCall, Variable};

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

impl Expression {
    pub fn identifier<S: Into<String>>(identifier: S) -> Self {
        Self::Identifier(identifier.into())
    }

    pub fn in_parentheses(self) -> Self {
        Self::Parenthese(self.into())
    }
}

impl From<bool> for Expression {
    fn from(boolean: bool) -> Expression {
        if boolean {
            Expression::True
        } else {
            Expression::False
        }
    }
}

impl From<f64> for Expression {
    fn from(value: f64) -> Expression {
        match value.classify() {
            FpCategory::Nan => BinaryExpression::new(
                BinaryOperator::Slash,
                DecimalNumber::new(0.0),
                DecimalNumber::new(0.0),
            )
            .into(),
            FpCategory::Infinite => BinaryExpression::new(
                BinaryOperator::Slash,
                Expression::from(if value.is_sign_positive() { 1.0 } else { -1.0 }),
                DecimalNumber::new(0.0),
            )
            .into(),
            FpCategory::Zero => DecimalNumber::new(0.0).into(),
            FpCategory::Subnormal | FpCategory::Normal => {
                if value < 0.0 {
                    UnaryExpression::new(UnaryOperator::Minus, Expression::from(value.abs())).into()
                } else if value < 0.1 {
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

impl From<BinaryNumber> for Expression {
    fn from(number: BinaryNumber) -> Self {
        Self::Number(NumberExpression::Binary(number))
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

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        match variable {
            Variable::Identifier(identifier) => Self::Identifier(identifier),
            Variable::Field(field) => Self::Field(field),
            Variable::Index(index) => Self::Index(index),
        }
    }
}
