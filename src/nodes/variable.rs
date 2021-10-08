use crate::nodes::{FieldExpression, Identifier, IndexExpression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variable {
    Identifier(Identifier),
    Field(Box<FieldExpression>),
    Index(Box<IndexExpression>),
}

impl Variable {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self::Identifier(Identifier::new(name))
    }
}

impl From<Identifier> for Variable {
    fn from(identifier: Identifier) -> Self {
        Self::Identifier(identifier)
    }
}

impl From<FieldExpression> for Variable {
    fn from(field: FieldExpression) -> Self {
        Self::Field(field.into())
    }
}

impl From<IndexExpression> for Variable {
    fn from(index: IndexExpression) -> Self {
        Self::Index(index.into())
    }
}
