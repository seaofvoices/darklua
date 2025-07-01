use crate::nodes::{FieldExpression, Identifier, IndexExpression};

/// Represents a variable reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variable {
    /// A simple named variable (e.g., `x`, `count`, `self`).
    Identifier(Identifier),
    /// A field access on a table or object using dot notation (e.g., `table.field`, `self.name`).
    Field(Box<FieldExpression>),
    /// An index access on a table using bracket notation (e.g., `array[1]`, `table[key]`).
    Index(Box<IndexExpression>),
}

impl Variable {
    /// Creates a new variable from an identifier name.
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
