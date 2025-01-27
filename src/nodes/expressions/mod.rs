mod binary;
mod field;
mod function;
mod if_expression;
mod index;
mod interpolated_string;
mod number;
mod parenthese;
mod prefix;
mod string;
pub(crate) mod string_utils;
mod table;
mod type_cast;
mod unary;

pub use binary::*;
pub use field::*;
pub use function::*;
pub use if_expression::*;
pub use index::*;
pub use interpolated_string::*;
pub use number::*;
pub use parenthese::*;
pub use prefix::*;
pub use string::*;
pub use string_utils::StringError;
pub use table::*;
pub use type_cast::*;
pub use unary::*;

use crate::nodes::{FunctionCall, Identifier, Token, Variable};

use super::impl_token_fns;

use std::num::FpCategory;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Binary(Box<BinaryExpression>),
    Call(Box<FunctionCall>),
    False(Option<Token>),
    Field(Box<FieldExpression>),
    Function(FunctionExpression),
    Identifier(Identifier),
    If(Box<IfExpression>),
    Index(Box<IndexExpression>),
    Nil(Option<Token>),
    Number(NumberExpression),
    Parenthese(Box<ParentheseExpression>),
    String(StringExpression),
    InterpolatedString(InterpolatedStringExpression),
    Table(TableExpression),
    True(Option<Token>),
    Unary(Box<UnaryExpression>),
    VariableArguments(Option<Token>),
    TypeCast(TypeCastExpression),
}

impl Expression {
    #[inline]
    pub fn nil() -> Self {
        Self::Nil(None)
    }

    #[inline]
    pub fn variable_arguments() -> Self {
        Self::VariableArguments(None)
    }

    pub fn identifier<S: Into<Identifier>>(identifier: S) -> Self {
        Self::Identifier(identifier.into())
    }

    pub fn in_parentheses(self) -> Self {
        Self::Parenthese(ParentheseExpression::new(self).into())
    }
}

impl From<bool> for Expression {
    fn from(boolean: bool) -> Expression {
        if boolean {
            Expression::True(None)
        } else {
            Expression::False(None)
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
            FpCategory::Zero => {
                DecimalNumber::new(if value.is_sign_positive() { 0.0 } else { -0.0 }).into()
            }
            FpCategory::Subnormal | FpCategory::Normal => {
                if value < 0.0 {
                    UnaryExpression::new(UnaryOperator::Minus, Expression::from(value.abs())).into()
                } else if value < 0.1 {
                    let exponent = value.log10().floor();

                    DecimalNumber::new(value)
                        .with_exponent(exponent as i64, true)
                        .into()
                } else if value > 999.0 && (value / 100.0).fract() == 0.0 {
                    let mut exponent = value.log10().floor();
                    let mut power = 10_f64.powf(exponent);

                    while exponent > 2.0 && (value / power).fract() != 0.0 {
                        exponent -= 1.0;
                        power /= 10.0;
                    }

                    DecimalNumber::new(value)
                        .with_exponent(exponent as i64, true)
                        .into()
                } else {
                    DecimalNumber::new(value).into()
                }
            }
        }
    }
}

impl From<f32> for Expression {
    fn from(value: f32) -> Self {
        (value as f64).into()
    }
}

impl From<usize> for Expression {
    fn from(value: usize) -> Self {
        (value as f64).into()
    }
}

impl From<u64> for Expression {
    fn from(value: u64) -> Self {
        (value as f64).into()
    }
}

impl From<u32> for Expression {
    fn from(value: u32) -> Self {
        (value as f64).into()
    }
}

impl From<u16> for Expression {
    fn from(value: u16) -> Self {
        (value as f64).into()
    }
}

impl From<u8> for Expression {
    fn from(value: u8) -> Self {
        (value as f64).into()
    }
}

impl From<i64> for Expression {
    fn from(value: i64) -> Self {
        (value as f64).into()
    }
}

impl From<i32> for Expression {
    fn from(value: i32) -> Self {
        (value as f64).into()
    }
}

impl From<i16> for Expression {
    fn from(value: i16) -> Self {
        (value as f64).into()
    }
}

impl From<i8> for Expression {
    fn from(value: i8) -> Self {
        (value as f64).into()
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

impl From<Identifier> for Expression {
    fn from(identifier: Identifier) -> Self {
        Expression::Identifier(identifier)
    }
}

impl From<IfExpression> for Expression {
    fn from(if_expression: IfExpression) -> Expression {
        Expression::If(Box::new(if_expression))
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
            Prefix::Parenthese(expression) => expression.into(),
        }
    }
}

impl From<ParentheseExpression> for Expression {
    fn from(expression: ParentheseExpression) -> Self {
        Self::Parenthese(expression.into())
    }
}

impl From<StringExpression> for Expression {
    fn from(string: StringExpression) -> Self {
        Self::String(string)
    }
}

impl From<InterpolatedStringExpression> for Expression {
    fn from(interpolated_string: InterpolatedStringExpression) -> Self {
        Self::InterpolatedString(interpolated_string)
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

impl From<TypeCastExpression> for Expression {
    fn from(type_cast: TypeCastExpression) -> Self {
        Self::TypeCast(type_cast)
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

impl<T: Into<Expression>> From<Option<T>> for Expression {
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::nil(),
            Some(value) => value.into(),
        }
    }
}

#[cfg(test)]
mod test {
    macro_rules! snapshot_from_expression {
        ($($name:ident => $input:expr),+ $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let result = crate::nodes::Expression::from($input);

                    insta::assert_debug_snapshot!(stringify!($name), result);
                }
            )+
        };
    }

    mod expression_from_floats {
        snapshot_from_expression!(
            f64_0 => 0_f64,
            f64_1e42 => 1e42_f64,
            f64_1_2345e50 => 1.2345e50_f64,
            f64_infinity => f64::INFINITY,
            i64_minus_one => -1_i64,
            f64_minus_zero => -0.0,
        );
    }
}
