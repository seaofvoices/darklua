use crate::nodes::{
    Block, FunctionBodyTokens, FunctionReturnType, FunctionVariadicType, GenericParameters,
    Identifier, Token, TypedIdentifier,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFunctionTokens {
    pub local: Token,
    pub function_body: FunctionBodyTokens,
}

impl LocalFunctionTokens {
    super::impl_token_fns!(target = [local, function_body]);
}

impl std::ops::Deref for LocalFunctionTokens {
    type Target = FunctionBodyTokens;

    fn deref(&self) -> &Self::Target {
        &self.function_body
    }
}

impl std::ops::DerefMut for LocalFunctionTokens {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.function_body
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFunctionStatement {
    identifier: Identifier,
    block: Block,
    parameters: Vec<TypedIdentifier>,
    is_variadic: bool,
    variadic_type: Option<FunctionVariadicType>,
    return_type: Option<FunctionReturnType>,
    generic_parameters: Option<GenericParameters>,
    tokens: Option<Box<LocalFunctionTokens>>,
}

impl LocalFunctionStatement {
    pub fn new(
        identifier: impl Into<Identifier>,
        block: Block,
        parameters: Vec<TypedIdentifier>,
        is_variadic: bool,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            block,
            parameters,
            is_variadic,
            variadic_type: None,
            return_type: None,
            generic_parameters: None,
            tokens: None,
        }
    }

    pub fn from_name(identifier: impl Into<Identifier>, block: impl Into<Block>) -> Self {
        Self {
            identifier: identifier.into(),
            block: block.into(),
            parameters: Vec::new(),
            is_variadic: false,
            variadic_type: None,
            return_type: None,
            generic_parameters: None,
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
        self.tokens.as_deref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut LocalFunctionTokens> {
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
    pub fn mutate_parameters(&mut self) -> &mut Vec<TypedIdentifier> {
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

    super::impl_token_fns!(
        target = [identifier]
        iter = [parameters, generic_parameters, tokens]
    );
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
