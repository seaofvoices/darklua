use crate::nodes::{Expression, Token, TypedIdentifier};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalAssignTokens {
    pub local: Token,
    pub equal: Option<Token>,
    pub variable_commas: Vec<Token>,
    pub value_commas: Vec<Token>,
}

impl LocalAssignTokens {
    pub fn clear_comments(&mut self) {
        self.local.clear_comments();
        self.variable_commas
            .iter_mut()
            .for_each(Token::clear_comments);
        self.value_commas.iter_mut().for_each(Token::clear_comments);
        if let Some(token) = &mut self.equal {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.local.clear_whitespaces();
        self.variable_commas
            .iter_mut()
            .for_each(Token::clear_whitespaces);
        self.value_commas
            .iter_mut()
            .for_each(Token::clear_whitespaces);
        if let Some(token) = &mut self.equal {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.local.replace_referenced_tokens(code);
        for comma in self.variable_commas.iter_mut() {
            comma.replace_referenced_tokens(code);
        }
        for comma in self.value_commas.iter_mut() {
            comma.replace_referenced_tokens(code);
        }
        if let Some(token) = &mut self.equal {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.local.shift_token_line(amount);
        for comma in self.variable_commas.iter_mut() {
            comma.shift_token_line(amount);
        }
        for comma in self.value_commas.iter_mut() {
            comma.shift_token_line(amount);
        }
        if let Some(token) = &mut self.equal {
            token.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalAssignStatement {
    variables: Vec<TypedIdentifier>,
    values: Vec<Expression>,
    tokens: Option<LocalAssignTokens>,
}

impl LocalAssignStatement {
    pub fn new(variables: Vec<TypedIdentifier>, values: Vec<Expression>) -> Self {
        Self {
            variables,
            values,
            tokens: None,
        }
    }

    pub fn from_variable<S: Into<TypedIdentifier>>(variable: S) -> Self {
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

    #[inline]
    pub fn get_tokens(&self) -> Option<&LocalAssignTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut LocalAssignTokens> {
        self.tokens.as_mut()
    }

    pub fn with_variable<S: Into<TypedIdentifier>>(mut self, variable: S) -> Self {
        self.variables.push(variable.into());
        self
    }

    pub fn with_value<E: Into<Expression>>(mut self, value: E) -> Self {
        self.values.push(value.into());
        self
    }

    pub fn into_assignments(self) -> (Vec<TypedIdentifier>, Vec<Expression>) {
        (self.variables, self.values)
    }

    pub fn append_assignment<S: Into<TypedIdentifier>>(&mut self, variable: S, value: Expression) {
        self.variables.push(variable.into());
        self.values.push(value);
    }

    pub fn for_each_assignment<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut TypedIdentifier, Option<&mut Expression>),
    {
        let mut values = self.values.iter_mut();
        self.variables
            .iter_mut()
            .for_each(|variable| callback(variable, values.next()));
    }

    #[inline]
    pub fn get_variables(&self) -> &Vec<TypedIdentifier> {
        &self.variables
    }

    #[inline]
    pub fn iter_variables(&self) -> impl Iterator<Item = &TypedIdentifier> {
        self.variables.iter()
    }

    #[inline]
    pub fn iter_mut_variables(&mut self) -> impl Iterator<Item = &mut TypedIdentifier> {
        self.variables.iter_mut()
    }

    #[inline]
    pub fn append_variables(&mut self, variables: &mut Vec<TypedIdentifier>) {
        self.variables.append(variables);
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
    pub fn iter_values(&self) -> impl Iterator<Item = &Expression> {
        self.values.iter()
    }

    #[inline]
    pub fn push_variable(&mut self, variable: impl Into<TypedIdentifier>) {
        self.variables.push(variable.into());
    }

    #[inline]
    pub fn push_value(&mut self, value: impl Into<Expression>) {
        self.values.push(value.into());
    }

    #[inline]
    pub fn append_values(&mut self, values: &mut Vec<Expression>) {
        self.values.append(values);
    }

    #[inline]
    pub fn last_value(&self) -> Option<&Expression> {
        self.values.last()
    }

    pub fn pop_value(&mut self) -> Option<Expression> {
        let value = self.values.pop();
        if let Some(tokens) = &mut self.tokens {
            let length = self.values.len();
            if length == 0 {
                if !tokens.value_commas.is_empty() {
                    tokens.value_commas.clear();
                }
                if tokens.equal.is_some() {
                    tokens.equal = None;
                }
            } else {
                tokens.value_commas.truncate(length.saturating_sub(1));
            }
        }
        value
    }

    pub fn remove_value(&mut self, index: usize) -> Option<Expression> {
        if index < self.values.len() {
            let value = self.values.remove(index);

            if let Some(tokens) = &mut self.tokens {
                if index < tokens.value_commas.len() {
                    tokens.value_commas.remove(index);
                }
                if self.values.is_empty() && tokens.equal.is_some() {
                    tokens.equal = None;
                }
            }

            Some(value)
        } else {
            None
        }
    }

    pub fn remove_variable(&mut self, index: usize) -> Option<TypedIdentifier> {
        let len = self.variables.len();

        if len > 1 && index < len {
            let variable = self.variables.remove(index);

            if let Some(tokens) = &mut self.tokens {
                if index < tokens.variable_commas.len() {
                    tokens.variable_commas.remove(index);
                }
            }

            Some(variable)
        } else {
            None
        }
    }

    #[inline]
    pub fn values_len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn variables_len(&self) -> usize {
        self.variables.len()
    }

    #[inline]
    pub fn has_values(&self) -> bool {
        !self.values.is_empty()
    }

    pub fn clear_types(&mut self) {
        for variable in &mut self.variables {
            variable.remove_type();
        }
    }

    pub fn clear_comments(&mut self) {
        self.variables
            .iter_mut()
            .for_each(TypedIdentifier::clear_comments);
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.variables
            .iter_mut()
            .for_each(TypedIdentifier::clear_whitespaces);
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        for variable in self.variables.iter_mut() {
            variable.replace_referenced_tokens(code);
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        for variable in self.variables.iter_mut() {
            variable.shift_token_line(amount);
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod pop_value {
        use super::*;

        #[test]
        fn removes_the_equal_sign() {
            let mut assign = LocalAssignStatement::from_variable("var")
                .with_value(true)
                .with_tokens(LocalAssignTokens {
                    local: Token::from_content("local"),
                    equal: Some(Token::from_content("=")),
                    variable_commas: Vec::new(),
                    value_commas: Vec::new(),
                });

            assign.pop_value();

            pretty_assertions::assert_eq!(
                assign,
                LocalAssignStatement::from_variable("var").with_tokens(LocalAssignTokens {
                    local: Token::from_content("local"),
                    equal: None,
                    variable_commas: Vec::new(),
                    value_commas: Vec::new(),
                })
            );
        }

        #[test]
        fn removes_the_last_comma_token() {
            let mut assign = LocalAssignStatement::from_variable("var")
                .with_value(true)
                .with_value(false)
                .with_tokens(LocalAssignTokens {
                    local: Token::from_content("local"),
                    equal: Some(Token::from_content("=")),
                    variable_commas: Vec::new(),
                    value_commas: vec![Token::from_content(",")],
                });

            assign.pop_value();

            pretty_assertions::assert_eq!(
                assign,
                LocalAssignStatement::from_variable("var")
                    .with_value(true)
                    .with_tokens(LocalAssignTokens {
                        local: Token::from_content("local"),
                        equal: Some(Token::from_content("=")),
                        variable_commas: Vec::new(),
                        value_commas: Vec::new(),
                    })
            );
        }

        #[test]
        fn removes_one_comma_token() {
            let mut assign = LocalAssignStatement::from_variable("var")
                .with_value(true)
                .with_value(false)
                .with_value(true)
                .with_tokens(LocalAssignTokens {
                    local: Token::from_content("local"),
                    equal: Some(Token::from_content("=")),
                    variable_commas: Vec::new(),
                    value_commas: vec![Token::from_content(","), Token::from_content(",")],
                });

            assign.pop_value();

            pretty_assertions::assert_eq!(
                assign,
                LocalAssignStatement::from_variable("var")
                    .with_value(true)
                    .with_value(false)
                    .with_tokens(LocalAssignTokens {
                        local: Token::from_content("local"),
                        equal: Some(Token::from_content("=")),
                        variable_commas: Vec::new(),
                        value_commas: vec![Token::from_content(",")],
                    })
            );
        }
    }

    mod remove_variable {
        use super::*;

        #[test]
        fn single_variable_returns_none_without_mutating() {
            let mut assign = LocalAssignStatement::from_variable("var").with_value(true);
            let copy = assign.clone();

            assert_eq!(assign.remove_variable(0), None);

            pretty_assertions::assert_eq!(assign, copy);
        }

        #[test]
        fn single_variable_remove_outside_of_bounds() {
            let mut assign = LocalAssignStatement::from_variable("var");
            let copy = assign.clone();

            assert_eq!(assign.remove_variable(1), None);
            pretty_assertions::assert_eq!(assign, copy);

            assert_eq!(assign.remove_variable(3), None);
            pretty_assertions::assert_eq!(assign, copy);
        }

        #[test]
        fn two_variables_remove_first() {
            let mut assign = LocalAssignStatement::from_variable("var")
                .with_variable("var2")
                .with_value(true)
                .with_value(false);

            assert_eq!(assign.remove_variable(0), Some(TypedIdentifier::new("var")));

            pretty_assertions::assert_eq!(
                assign,
                LocalAssignStatement::from_variable("var2")
                    .with_value(true)
                    .with_value(false)
            );
        }

        #[test]
        fn two_variables_remove_second() {
            let mut assign = LocalAssignStatement::from_variable("var")
                .with_variable("var2")
                .with_value(true)
                .with_value(false);

            assert_eq!(
                assign.remove_variable(1),
                Some(TypedIdentifier::new("var2"))
            );

            pretty_assertions::assert_eq!(
                assign,
                LocalAssignStatement::from_variable("var")
                    .with_value(true)
                    .with_value(false)
            );
        }
    }
}
