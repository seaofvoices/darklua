use crate::nodes::{Expression, Token, Variable};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssignTokens {
    pub equal: Token,
    pub variable_commas: Vec<Token>,
    pub value_commas: Vec<Token>,
}

impl AssignTokens {
    super::impl_token_fns!(
        target = [equal]
        iter = [variable_commas, value_commas]
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssignStatement {
    variables: Vec<Variable>,
    values: Vec<Expression>,
    tokens: Option<AssignTokens>,
}

impl AssignStatement {
    pub fn new(variables: Vec<Variable>, values: Vec<Expression>) -> Self {
        Self {
            variables,
            values,
            tokens: None,
        }
    }

    pub fn from_variable<V: Into<Variable>, E: Into<Expression>>(variable: V, value: E) -> Self {
        Self {
            variables: vec![variable.into()],
            values: vec![value.into()],
            tokens: None,
        }
    }

    #[inline]
    pub fn variables_len(&self) -> usize {
        self.variables.len()
    }

    #[inline]
    pub fn values_len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn get_variables(&self) -> &Vec<Variable> {
        &self.variables
    }

    #[inline]
    pub fn iter_variables(&self) -> impl Iterator<Item = &Variable> {
        self.variables.iter()
    }

    #[inline]
    pub fn iter_mut_variables(&mut self) -> impl Iterator<Item = &mut Variable> {
        self.variables.iter_mut()
    }

    #[inline]
    pub fn last_value(&self) -> Option<&Expression> {
        self.values.last()
    }

    #[inline]
    pub fn iter_values(&self) -> impl Iterator<Item = &Expression> {
        self.values.iter()
    }

    #[inline]
    pub fn iter_mut_values(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.values.iter_mut()
    }

    #[inline]
    pub fn mutate_variables(&mut self) -> &mut Vec<Variable> {
        &mut self.variables
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

    pub fn with_tokens(mut self, tokens: AssignTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: AssignTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&AssignTokens> {
        self.tokens.as_ref()
    }

    super::impl_token_fns!(iter = [tokens]);
}
