use crate::nodes::{Block, Identifier, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionNameTokens {
    pub periods: Vec<Token>,
    pub colon: Option<Token>,
}

impl FunctionNameTokens {
    pub fn clear_comments(&mut self) {
        self.periods.iter_mut().for_each(Token::clear_comments);
        if let Some(token) = &mut self.colon {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.periods.iter_mut().for_each(Token::clear_whitespaces);
        if let Some(token) = &mut self.colon {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        for token in self.periods.iter_mut() {
            token.replace_referenced_tokens(code);
        }
        if let Some(token) = &mut self.colon {
            token.replace_referenced_tokens(code);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionName {
    name: Identifier,
    field_names: Vec<Identifier>,
    method: Option<Identifier>,
    tokens: Option<FunctionNameTokens>,
}

impl FunctionName {
    pub fn new(name: Identifier, field_names: Vec<Identifier>, method: Option<Identifier>) -> Self {
        Self {
            name,
            field_names,
            method,
            tokens: None,
        }
    }

    pub fn from_name<S: Into<Identifier>>(name: S) -> Self {
        Self {
            name: name.into(),
            field_names: Vec::new(),
            method: None,
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: FunctionNameTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: FunctionNameTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&FunctionNameTokens> {
        self.tokens.as_ref()
    }

    pub fn with_field<S: Into<Identifier>>(mut self, field: S) -> Self {
        self.field_names.push(field.into());
        self
    }

    pub fn with_fields(mut self, field_names: Vec<Identifier>) -> Self {
        self.field_names = field_names;
        self
    }

    pub fn with_method<S: Into<Identifier>>(mut self, method: S) -> Self {
        self.method.replace(method.into());
        self
    }

    pub fn push_field<S: Into<Identifier>>(&mut self, field: S) {
        self.field_names.push(field.into());
    }

    #[inline]
    pub fn remove_method(&mut self) -> Option<Identifier> {
        self.method.take()
    }

    #[inline]
    pub fn get_method(&self) -> Option<&Identifier> {
        self.method.as_ref()
    }

    #[inline]
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: Identifier) {
        self.name = name;
    }

    #[inline]
    pub fn get_field_names(&self) -> &Vec<Identifier> {
        &self.field_names
    }

    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut Identifier {
        &mut self.name
    }

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
        for field in self.field_names.iter_mut() {
            field.clear_comments();
        }
        if let Some(method) = &mut self.method {
            method.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
        for field in self.field_names.iter_mut() {
            field.clear_whitespaces();
        }
        if let Some(method) = &mut self.method {
            method.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
        for field in self.field_names.iter_mut() {
            field.replace_referenced_tokens(code);
        }
        if let Some(method) = &mut self.method {
            method.replace_referenced_tokens(code);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionStatementTokens {
    pub function: Token,
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
    pub end: Token,
    pub parameter_commas: Vec<Token>,
    pub variable_arguments: Option<Token>,
}

impl FunctionStatementTokens {
    pub fn clear_comments(&mut self) {
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionStatement {
    name: FunctionName,
    block: Block,
    parameters: Vec<Identifier>,
    is_variadic: bool,
    tokens: Option<Box<FunctionStatementTokens>>,
}

impl FunctionStatement {
    pub fn new(
        name: FunctionName,
        block: Block,
        parameters: Vec<Identifier>,
        is_variadic: bool,
    ) -> Self {
        Self {
            name,
            block,
            parameters,
            is_variadic,
            tokens: None,
        }
    }

    pub fn from_name<S: Into<String>, B: Into<Block>>(name: S, block: B) -> Self {
        Self {
            name: FunctionName::from_name(name),
            block: block.into(),
            parameters: Vec::new(),
            is_variadic: false,
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: FunctionStatementTokens) -> Self {
        self.tokens = Some(tokens.into());
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: FunctionStatementTokens) {
        self.tokens = Some(tokens.into());
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&FunctionStatementTokens> {
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
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_name(&self) -> &FunctionName {
        &self.name
    }

    #[inline]
    pub fn parameters_count(&self) -> usize {
        self.parameters.len()
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
    pub fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn mutate_function_name(&mut self) -> &mut FunctionName {
        &mut self.name
    }

    #[inline]
    pub fn mutate_parameters(&mut self) -> &mut Vec<Identifier> {
        &mut self.parameters
    }

    pub fn remove_method(&mut self) {
        if let Some(method_name) = self.name.remove_method() {
            self.name.push_field(method_name);
            self.parameters.insert(0, Identifier::new("self"));
        }
    }

    #[inline]
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    pub fn clear_comments(&mut self) {
        self.name.clear_comments();
        self.parameters
            .iter_mut()
            .for_each(Identifier::clear_comments);
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.name.clear_whitespaces();
        self.parameters
            .iter_mut()
            .for_each(Identifier::clear_whitespaces);
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.name.replace_referenced_tokens(code);
        for parameter in self.parameters.iter_mut() {
            parameter.replace_referenced_tokens(code);
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }
}
