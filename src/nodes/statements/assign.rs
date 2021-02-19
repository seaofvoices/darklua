use crate::nodes::{
    Expression,
    FieldExpression,
    IndexExpression,
    Prefix,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variable {
    Identifier(String),
    Field(Box<FieldExpression>),
    Index(Box<IndexExpression>),
}

fn get_root_identifier_from_expression(expression: &Expression) -> Option<&String> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier),
        Expression::Field(field) => {
            get_root_identifier_from_prefix(field.get_prefix())
        }
        Expression::Index(index) => {
            get_root_identifier_from_prefix(index.get_prefix())
        }
        Expression::Parenthese(expression) => {
            get_root_identifier_from_expression(expression)
        }
        _ => None,
    }
}

fn get_root_identifier_from_prefix(prefix: &Prefix) -> Option<&String> {
    match prefix {
        Prefix::Identifier(identifier) => Some(identifier),
        Prefix::Field(field) => {
            get_root_identifier_from_prefix(field.get_prefix())
        }
        Prefix::Index(index) => {
            get_root_identifier_from_prefix(index.get_prefix())
        }
        Prefix::Parenthese(expression) => {
            get_root_identifier_from_expression(expression)
        }
        Prefix::Call(_) => None,
    }
}

impl Variable {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self::Identifier(name.into())
    }

    pub fn get_root_identifier(&self) -> Option<&String> {
        match self {
            Self::Identifier(identifier) => Some(identifier),
            Self::Field(field) => {
                get_root_identifier_from_prefix(field.get_prefix())
            }
            Self::Index(index) => {
                get_root_identifier_from_prefix(index.get_prefix())
            }
        }
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
