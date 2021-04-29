use crate::nodes::{
    Expression,
    FieldExpression,
    IndexExpression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variable {
    Identifier(String),
    Field(Box<FieldExpression>),
    Index(Box<IndexExpression>),
}

impl Variable {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self::Identifier(name.into())
    }
}

impl From<FieldExpression> for Variable {
    fn from(field: FieldExpression) -> Self {
        Variable::Field(Box::new(field))
    }
}

impl From<IndexExpression> for Variable {
    fn from(index: IndexExpression) -> Self {
        Variable::Index(Box::new(index))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssignStatement {
    variables: Vec<Variable>,
    values: Vec<Expression>,
}

impl AssignStatement {
    pub fn new(variables: Vec<Variable>, values: Vec<Expression>) -> Self {
        Self {
            variables,
            values,
        }
    }

    #[inline]
    pub fn get_variables(&self) -> &Vec<Variable> {
        &self.variables
    }

    #[inline]
    pub fn get_values(&self) -> &Vec<Expression> {
        &self.values
    }

    #[inline]
    pub fn mutate_variables(&mut self) -> &mut Vec<Variable> {
        &mut self.variables
    }

    #[inline]
    pub fn mutate_values(&mut self) -> &mut Vec<Expression> {
        &mut self.values
    }
}
