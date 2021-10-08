use crate::nodes::{Expression, Identifier, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalAssignTokens {
    pub local: Token,
    pub equal: Option<Token>,
    pub variable_commas: Vec<Token>,
    pub value_commas: Vec<Token>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalAssignStatement {
    variables: Vec<Identifier>,
    values: Vec<Expression>,
    tokens: Option<LocalAssignTokens>,
}

impl LocalAssignStatement {
    pub fn new(variables: Vec<Identifier>, values: Vec<Expression>) -> Self {
        Self {
            variables,
            values,
            tokens: None,
        }
    }

    pub fn from_variable<S: Into<Identifier>>(variable: S) -> Self {
        Self {
            variables: vec![variable.into()],
            values: Vec::new(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: LocalAssignTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: LocalAssignTokens) {
        self.tokens = Some(tokens);
    }

    pub fn with_variable<S: Into<Identifier>>(mut self, variable: S) -> Self {
        self.variables.push(variable.into());
        self
    }

    pub fn with_value<E: Into<Expression>>(mut self, value: E) -> Self {
        self.values.push(value.into());
        self
    }

    pub fn into_assignments(self) -> (Vec<Identifier>, Vec<Expression>) {
        (self.variables, self.values)
    }

    pub fn append_assignment<S: Into<Identifier>>(&mut self, variable: S, value: Expression) {
        self.variables.push(variable.into());
        self.values.push(value);
    }

    pub fn for_each_assignment<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Identifier, Option<&mut Expression>),
    {
        let mut values = self.values.iter_mut();
        self.variables
            .iter_mut()
            .for_each(|variable| callback(variable, values.next()));
    }

    #[inline]
    pub fn get_variables(&self) -> &Vec<Identifier> {
        &self.variables
    }

    #[inline]
    pub fn append_variables(&mut self, variables: &mut Vec<Identifier>) {
        self.variables.append(variables);
    }

    #[inline]
    pub fn get_values(&self) -> &Vec<Expression> {
        &self.values
    }

    #[inline]
    pub fn mutate_values(&mut self) -> &mut Vec<Expression> {
        &mut self.values
    }

    #[inline]
    pub fn extend_values<T: IntoIterator<Item = Expression>>(&mut self, iter: T) {
        self.values.extend(iter);
    }

    #[inline]
    pub fn iter_mut_values(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.values.iter_mut()
    }

    #[inline]
    pub fn append_values(&mut self, values: &mut Vec<Expression>) {
        self.values.append(values);
    }

    #[inline]
    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }
}
