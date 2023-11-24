use crate::nodes::{
    Expression, FieldExpression, FunctionCall, Identifier, IndexExpression, ParentheseExpression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prefix {
    Call(FunctionCall),
    Field(Box<FieldExpression>),
    Identifier(Identifier),
    Index(Box<IndexExpression>),
    Parenthese(ParentheseExpression),
}

impl Prefix {
    pub fn from_name<S: Into<Identifier>>(name: S) -> Self {
        Self::Identifier(name.into())
    }
}

impl From<Expression> for Prefix {
    fn from(expression: Expression) -> Self {
        match expression {
            Expression::Call(call) => Prefix::Call(*call),
            Expression::Field(field) => Prefix::Field(field),
            Expression::Identifier(identifier) => Prefix::Identifier(identifier),
            Expression::Index(index) => Prefix::Index(index),
            Expression::Parenthese(parenthese) => Prefix::Parenthese(*parenthese),
            Expression::Binary(_)
            | Expression::False(_)
            | Expression::Function(_)
            | Expression::If(_)
            | Expression::Nil(_)
            | Expression::Number(_)
            | Expression::String(_)
            | Expression::InterpolatedString(_)
            | Expression::Table(_)
            | Expression::True(_)
            | Expression::TypeCast(_)
            | Expression::Unary(_)
            | Expression::VariableArguments(_) => {
                Prefix::Parenthese(ParentheseExpression::new(expression))
            }
        }
    }
}

impl From<FunctionCall> for Prefix {
    fn from(call: FunctionCall) -> Self {
        Self::Call(call)
    }
}

impl From<FieldExpression> for Prefix {
    fn from(field: FieldExpression) -> Self {
        Self::Field(field.into())
    }
}

impl From<Box<FieldExpression>> for Prefix {
    fn from(field: Box<FieldExpression>) -> Self {
        Self::Field(field)
    }
}

impl From<Identifier> for Prefix {
    fn from(identifier: Identifier) -> Self {
        Self::Identifier(identifier)
    }
}

impl From<IndexExpression> for Prefix {
    fn from(index: IndexExpression) -> Self {
        Self::Index(index.into())
    }
}

impl From<Box<IndexExpression>> for Prefix {
    fn from(index: Box<IndexExpression>) -> Self {
        Self::Index(index)
    }
}

impl From<ParentheseExpression> for Prefix {
    fn from(expression: ParentheseExpression) -> Self {
        Self::Parenthese(expression)
    }
}
