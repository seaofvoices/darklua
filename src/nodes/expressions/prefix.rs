use crate::nodes::{Expression, FieldExpression, FunctionCall, IndexExpression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prefix {
    Call(FunctionCall),
    Field(Box<FieldExpression>),
    Identifier(String),
    Index(Box<IndexExpression>),
    Parenthese(Expression),
}

impl Prefix {
    pub fn from_name<S: Into<String>>(name: S) -> Self {
        Self::Identifier(name.into())
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
        Self::Field(field.into())
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
