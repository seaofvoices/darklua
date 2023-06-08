use crate::nodes::{Block, Identifier, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFunctionTokens {
    pub local: Token,
    pub function: Token,
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
    pub end: Token,
    pub parameter_commas: Vec<Token>,
    pub variable_arguments: Option<Token>,
}

impl LocalFunctionTokens {
    pub fn clear_comments(&mut self) {
        self.local.clear_comments();
        self.function.clear_comments();
        self.opening_parenthese.clear_comments();
        self.closing_parenthese.clear_comments();
        self.end.clear_comments();
        self.parameter_commas
            .iter_mut()
            .for_each(Token::clear_comments);
        if let Some(token) = &mut self.variable_arguments {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.local.clear_whitespaces();
        self.function.clear_whitespaces();
        self.opening_parenthese.clear_whitespaces();
        self.closing_parenthese.clear_whitespaces();
        self.end.clear_whitespaces();
        self.parameter_commas
            .iter_mut()
            .for_each(Token::clear_whitespaces);
        if let Some(token) = &mut self.variable_arguments {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.local.replace_referenced_tokens(code);
        self.function.replace_referenced_tokens(code);
        self.opening_parenthese.replace_referenced_tokens(code);
        self.closing_parenthese.replace_referenced_tokens(code);
        self.end.replace_referenced_tokens(code);
        for comma in self.parameter_commas.iter_mut() {
            comma.replace_referenced_tokens(code);
        }
        if let Some(token) = &mut self.variable_arguments {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.local.shift_token_line(amount);
        self.function.shift_token_line(amount);
        self.opening_parenthese.shift_token_line(amount);
        self.closing_parenthese.shift_token_line(amount);
        self.end.shift_token_line(amount);
        for comma in self.parameter_commas.iter_mut() {
            comma.shift_token_line(amount);
        }
        if let Some(token) = &mut self.variable_arguments {
            token.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFunctionStatement {
    identifier: Identifier,
    block: Block,
    parameters: Vec<Identifier>,
    is_variadic: bool,
    tokens: Option<Box<LocalFunctionTokens>>,
}

impl LocalFunctionStatement {
    pub fn new(
        identifier: Identifier,
        block: Block,
        parameters: Vec<Identifier>,
        is_variadic: bool,
    ) -> Self {
        Self {
            identifier,
            block,
            parameters,
            is_variadic,
            tokens: None,
        }
    }

    pub fn from_name<S: Into<Identifier>, B: Into<Block>>(identifier: S, block: B) -> Self {
        Self {
            identifier: identifier.into(),
            block: block.into(),
            parameters: Vec::new(),
            is_variadic: false,
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: LocalFunctionTokens) -> Self {
        self.tokens = Some(tokens.into());
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: LocalFunctionTokens) {
        self.tokens = Some(tokens.into());
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&LocalFunctionTokens> {
        self.tokens.as_ref().map(|tokens| tokens.as_ref())
    }

    pub fn with_parameter<S: Into<Identifier>>(mut self, parameter: S) -> Self {
        self.parameters.push(parameter.into());
        self
    }

    pub fn variadic(mut self) -> Self {
        self.is_variadic = true;
        self
    }

    #[inline]
    pub fn mutate_parameters(&mut self) -> &mut Vec<Identifier> {
        &mut self.parameters
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut Identifier {
        &mut self.identifier
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_parameters(&self) -> &Vec<Identifier> {
        &self.parameters
    }

    #[inline]
    pub fn iter_parameters(&self) -> impl Iterator<Item = &Identifier> {
        self.parameters.iter()
    }

    #[inline]
    pub fn get_identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn get_name(&self) -> &str {
        self.identifier.get_name()
    }

    #[inline]
    pub fn has_parameter(&self, name: &str) -> bool {
        self.parameters
            .iter()
            .any(|parameter| parameter.get_name() == name)
    }

    #[inline]
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    #[inline]
    pub fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    #[inline]
    pub fn parameters_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn clear_comments(&mut self) {
        self.identifier.clear_comments();
        self.parameters
            .iter_mut()
            .for_each(Identifier::clear_comments);
        if let Some(tokens) = self.tokens.as_mut() {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.identifier.clear_whitespaces();
        self.parameters
            .iter_mut()
            .for_each(Identifier::clear_whitespaces);
        if let Some(tokens) = self.tokens.as_mut() {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.identifier.replace_referenced_tokens(code);
        for parameter in self.parameters.iter_mut() {
            parameter.replace_referenced_tokens(code);
        }
        if let Some(tokens) = self.tokens.as_mut() {
            tokens.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.identifier.shift_token_line(amount);
        for parameter in self.parameters.iter_mut() {
            parameter.shift_token_line(amount);
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn has_parameter_is_true_when_single_param_matches() {
        let func = LocalFunctionStatement::from_name("foo", Block::default()).with_parameter("bar");

        assert!(func.has_parameter("bar"));
    }

    #[test]
    fn has_parameter_is_true_when_at_least_one_param_matches() {
        let func = LocalFunctionStatement::from_name("foo", Block::default())
            .with_parameter("bar")
            .with_parameter("baz");

        assert!(func.has_parameter("baz"));
    }

    #[test]
    fn has_parameter_is_false_when_none_matches() {
        let func = LocalFunctionStatement::from_name("foo", Block::default())
            .with_parameter("bar")
            .with_parameter("baz");

        assert!(!func.has_parameter("foo"));
    }
}
