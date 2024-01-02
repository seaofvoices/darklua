use crate::nodes::{
    Block, FunctionBodyTokens, FunctionReturnType, FunctionVariadicType, GenericParameters,
    Identifier, Token, TypedIdentifier,
};

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

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        for token in self.periods.iter_mut() {
            token.shift_token_line(amount);
        }
        if let Some(token) = &mut self.colon {
            token.shift_token_line(amount);
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
    pub fn has_method(&self) -> bool {
        self.method.is_some()
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

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
        for field in self.field_names.iter_mut() {
            field.shift_token_line(amount);
        }
        if let Some(method) = &mut self.method {
            method.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionStatement {
    name: FunctionName,
    block: Block,
    parameters: Vec<TypedIdentifier>,
    is_variadic: bool,
    variadic_type: Option<FunctionVariadicType>,
    return_type: Option<FunctionReturnType>,
    generic_parameters: Option<GenericParameters>,
    tokens: Option<Box<FunctionBodyTokens>>,
}

impl FunctionStatement {
    pub fn new(
        name: FunctionName,
        block: Block,
        parameters: Vec<TypedIdentifier>,
        is_variadic: bool,
    ) -> Self {
        Self {
            name,
            block,
            parameters,
            is_variadic,
            variadic_type: None,
            return_type: None,
            generic_parameters: None,
            tokens: None,
        }
    }

    pub fn from_name<S: Into<String>, B: Into<Block>>(name: S, block: B) -> Self {
        Self {
            name: FunctionName::from_name(name),
            block: block.into(),
            parameters: Vec::new(),
            is_variadic: false,
            variadic_type: None,
            return_type: None,
            generic_parameters: None,
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: FunctionBodyTokens) -> Self {
        self.tokens = Some(tokens.into());
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: FunctionBodyTokens) {
        self.tokens = Some(tokens.into());
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&FunctionBodyTokens> {
        self.tokens.as_deref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut FunctionBodyTokens> {
        self.tokens.as_deref_mut()
    }

    pub fn with_parameter(mut self, parameter: impl Into<TypedIdentifier>) -> Self {
        self.parameters.push(parameter.into());
        self
    }

    pub fn variadic(mut self) -> Self {
        self.is_variadic = true;
        self
    }

    pub fn with_variadic_type(mut self, r#type: impl Into<FunctionVariadicType>) -> Self {
        self.is_variadic = true;
        self.variadic_type = Some(r#type.into());
        self
    }

    pub fn set_variadic_type(&mut self, r#type: impl Into<FunctionVariadicType>) {
        self.is_variadic = true;
        self.variadic_type = Some(r#type.into());
    }

    #[inline]
    pub fn get_variadic_type(&self) -> Option<&FunctionVariadicType> {
        self.variadic_type.as_ref()
    }

    #[inline]
    pub fn has_variadic_type(&self) -> bool {
        self.variadic_type.is_some()
    }

    #[inline]
    pub fn mutate_variadic_type(&mut self) -> Option<&mut FunctionVariadicType> {
        self.variadic_type.as_mut()
    }

    pub fn with_return_type(mut self, return_type: impl Into<FunctionReturnType>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    pub fn set_return_type(&mut self, return_type: impl Into<FunctionReturnType>) {
        self.return_type = Some(return_type.into());
    }

    #[inline]
    pub fn get_return_type(&self) -> Option<&FunctionReturnType> {
        self.return_type.as_ref()
    }

    #[inline]
    pub fn has_return_type(&self) -> bool {
        self.return_type.is_some()
    }

    #[inline]
    pub fn mutate_return_type(&mut self) -> Option<&mut FunctionReturnType> {
        self.return_type.as_mut()
    }

    pub fn with_generic_parameters(mut self, generic_parameters: GenericParameters) -> Self {
        self.generic_parameters = Some(generic_parameters);
        self
    }

    #[inline]
    pub fn set_generic_parameters(&mut self, generic_parameters: GenericParameters) {
        self.generic_parameters = Some(generic_parameters);
    }

    #[inline]
    pub fn get_generic_parameters(&self) -> Option<&GenericParameters> {
        self.generic_parameters.as_ref()
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
    pub fn get_parameters(&self) -> &Vec<TypedIdentifier> {
        &self.parameters
    }

    #[inline]
    pub fn iter_parameters(&self) -> impl Iterator<Item = &TypedIdentifier> {
        self.parameters.iter()
    }

    #[inline]
    pub fn iter_mut_parameters(&mut self) -> impl Iterator<Item = &mut TypedIdentifier> {
        self.parameters.iter_mut()
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
    pub fn mutate_parameters(&mut self) -> &mut Vec<TypedIdentifier> {
        &mut self.parameters
    }

    pub fn remove_method(&mut self) {
        if let Some(method_name) = self.name.remove_method() {
            self.name.push_field(method_name);
            self.parameters.insert(0, TypedIdentifier::new("self"));
        }
    }

    #[inline]
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    pub fn clear_types(&mut self) {
        self.return_type.take();
        self.variadic_type.take();
        self.generic_parameters.take();
        for parameter in &mut self.parameters {
            parameter.remove_type();
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.variable_arguments_colon.take();
        }
    }

    pub fn clear_comments(&mut self) {
        self.name.clear_comments();
        self.parameters
            .iter_mut()
            .for_each(TypedIdentifier::clear_comments);
        if let Some(generics) = &mut self.generic_parameters {
            generics.clear_comments();
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.name.clear_whitespaces();
        self.parameters
            .iter_mut()
            .for_each(TypedIdentifier::clear_whitespaces);
        if let Some(generics) = &mut self.generic_parameters {
            generics.clear_whitespaces();
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.name.replace_referenced_tokens(code);
        for parameter in self.parameters.iter_mut() {
            parameter.replace_referenced_tokens(code);
        }
        if let Some(generics) = &mut self.generic_parameters {
            generics.replace_referenced_tokens(code);
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.name.shift_token_line(amount);
        for parameter in self.parameters.iter_mut() {
            parameter.shift_token_line(amount);
        }
        if let Some(generics) = &mut self.generic_parameters {
            generics.shift_token_line(amount);
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
    }
}
