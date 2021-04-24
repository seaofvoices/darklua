use crate::nodes::{
    Expression,
    FieldExpression,
    FunctionCall,
    IndexExpression,
};

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

impl From<FieldExpression> for Prefix {
    fn from(field: FieldExpression) -> Self {
        Prefix::Field(Box::new(field))
    }
}
