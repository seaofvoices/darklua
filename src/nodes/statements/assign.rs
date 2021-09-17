use crate::nodes::{Expression, FieldExpression, IndexExpression};

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
        Self::Field(field.into())
    }
}

impl From<IndexExpression> for Variable {
    fn from(index: IndexExpression) -> Self {
        Self::Index(index.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssignStatement {
    variables: Vec<Variable>,
    values: Vec<Expression>,
}

impl AssignStatement {
    pub fn new(variables: Vec<Variable>, values: Vec<Expression>) -> Self {
        Self { variables, values }
    }

    pub fn from_variable<V: Into<Variable>, E: Into<Expression>>(variable: V, value: E) -> Self {
        Self {
            variables: vec![variable.into()],
            values: vec![value.into()],
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

    pub fn append_assignment<V: Into<Variable>, E: Into<Expression>>(
        mut self,
        variable: V,
        value: E,
    ) -> Self {
        self.variables.push(variable.into());
        self.values.push(value.into());
        self
    }
}
