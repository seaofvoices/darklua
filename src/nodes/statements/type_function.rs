use crate::nodes::{
    Block, FunctionBodyTokens, FunctionReturnType, FunctionVariadicType, GenericParameters,
    Identifier, Token, TypedIdentifier,
};

/// Represents a type function statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeFunctionStatement {
    identifier: Identifier,
    block: Block,
    parameters: Vec<TypedIdentifier>,
    is_variadic: bool,
    variadic_type: Option<FunctionVariadicType>,
    return_type: Option<FunctionReturnType>,
    generic_parameters: Option<GenericParameters>,
    exported: bool,
    tokens: Option<TypeFunctionStatementTokens>,
}

impl TypeFunctionStatement {
    /// Creates a new type function statement.
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
            exported: false,
            tokens: None,
        }
    }

    /// Creates a new type function statement with a given name and block.
    pub fn from_name(identifier: impl Into<Identifier>, block: impl Into<Block>) -> Self {
        Self {
            identifier: identifier.into(),
            block: block.into(),
            parameters: Vec::new(),
            is_variadic: false,
            variadic_type: None,
            return_type: None,
            generic_parameters: None,
            exported: false,
            tokens: None,
        }
    }

    /// Adds a parameter to this type function.
    pub fn with_parameter(mut self, parameter: impl Into<TypedIdentifier>) -> Self {
        self.parameters.push(parameter.into());
        self
    }

    /// Marks this function as variadic.
    pub fn variadic(mut self) -> Self {
        self.is_variadic = true;
        self
    }

    /// Returns whether this type function is variadic.
    pub fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    /// Sets the variadic type for this function.
    ///
    /// If the function is not already variadic, this will make it variadic.
    pub fn with_variadic_type(mut self, r#type: impl Into<FunctionVariadicType>) -> Self {
        self.is_variadic = true;
        self.variadic_type = Some(r#type.into());
        self
    }

    /// Sets the variadic type for this function.
    ///
    /// If the function is not already variadic, this will make it variadic.
    pub fn set_variadic_type(&mut self, r#type: impl Into<FunctionVariadicType>) {
        self.is_variadic = true;
        self.variadic_type = Some(r#type.into());
    }

    /// Returns the variadic type, if any.
    pub fn get_variadic_type(&self) -> Option<&FunctionVariadicType> {
        self.variadic_type.as_ref()
    }

    /// Returns whether this function has a variadic type.
    pub fn has_variadic_type(&self) -> bool {
        self.variadic_type.is_some()
    }

    /// Returns a mutable reference to the variadic type, if any.
    pub fn mutate_variadic_type(&mut self) -> Option<&mut FunctionVariadicType> {
        self.variadic_type.as_mut()
    }

    /// Sets the return type for this function.
    pub fn with_return_type(mut self, return_type: impl Into<FunctionReturnType>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    /// Sets the return type for this function.
    pub fn set_return_type(&mut self, return_type: impl Into<FunctionReturnType>) {
        self.return_type = Some(return_type.into());
    }

    /// Returns the return type, if any.
    pub fn get_return_type(&self) -> Option<&FunctionReturnType> {
        self.return_type.as_ref()
    }

    /// Returns whether this function has a return type.
    pub fn has_return_type(&self) -> bool {
        self.return_type.is_some()
    }

    /// Returns a mutable reference to the return type, if any.
    pub fn mutate_return_type(&mut self) -> Option<&mut FunctionReturnType> {
        self.return_type.as_mut()
    }

    /// Sets the generic parameters for this function.
    pub fn with_generic_parameters(mut self, generic_parameters: GenericParameters) -> Self {
        self.generic_parameters = Some(generic_parameters);
        self
    }

    /// Sets the generic parameters for this function.
    pub fn set_generic_parameters(&mut self, generic_parameters: GenericParameters) {
        self.generic_parameters = Some(generic_parameters);
    }

    /// Returns the generic parameters, if any.
    pub fn get_generic_parameters(&self) -> Option<&GenericParameters> {
        self.generic_parameters.as_ref()
    }

    /// Returns a mutable reference to the parameters.
    pub fn mutate_parameters(&mut self) -> &mut Vec<TypedIdentifier> {
        &mut self.parameters
    }

    /// Returns a mutable reference to the block.
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    /// Returns a mutable reference to the identifier.
    pub fn mutate_identifier(&mut self) -> &mut Identifier {
        &mut self.identifier
    }

    /// Returns the function's block.
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    /// Returns the function's parameters.
    pub fn get_parameters(&self) -> &Vec<TypedIdentifier> {
        &self.parameters
    }

    /// Returns the number of parameters.
    pub fn parameters_count(&self) -> usize {
        self.parameters.len()
    }

    /// Returns an iterator over the parameters.
    pub fn iter_parameters(&self) -> impl Iterator<Item = &TypedIdentifier> {
        self.parameters.iter()
    }

    /// Returns a mutable iterator over the parameters.
    pub fn iter_mut_parameters(&mut self) -> impl Iterator<Item = &mut TypedIdentifier> {
        self.parameters.iter_mut()
    }

    /// Returns whether this type function has a parameter with the given name.
    pub fn has_parameter(&self, name: &str) -> bool {
        self.parameters
            .iter()
            .any(|parameter| parameter.get_name() == name)
    }

    /// Returns whether this type function has parameters.
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Returns the function's identifier.
    pub fn get_identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// Marks this type function statement as exported.
    pub fn export(mut self) -> Self {
        self.exported = true;
        self
    }

    /// Marks this type function statement as exported.
    pub fn set_exported(&mut self) {
        self.exported = true;
    }

    /// Removes the exported status from this type function statement.
    pub fn remove_exported(&mut self) {
        self.exported = false;
        if let Some(tokens) = self.tokens.as_mut() {
            tokens.export.take();
        }
    }

    /// Returns whether this type function is exported.
    pub fn is_exported(&self) -> bool {
        self.exported
    }

    /// Sets the tokens for this type function statement.
    pub fn with_tokens(mut self, tokens: TypeFunctionStatementTokens) -> Self {
        self.tokens = Some(tokens.into());
        self
    }

    /// Sets the tokens for this type function statement.
    pub fn set_tokens(&mut self, tokens: TypeFunctionStatementTokens) {
        self.tokens = Some(tokens.into());
    }

    /// Returns the tokens for this type function statement, if any.
    pub fn get_tokens(&self) -> Option<&TypeFunctionStatementTokens> {
        self.tokens.as_ref()
    }

    /// Returns a mutable reference to the tokens, if any.
    pub fn mutate_tokens(&mut self) -> Option<&mut TypeFunctionStatementTokens> {
        self.tokens.as_mut()
    }

    /// Returns a mutable reference to the first token for this statement, creating it if missing.
    pub fn mutate_first_token(&mut self) -> &mut Token {
        self.set_default_tokens();
        let tokens = self.tokens.as_mut().unwrap();
        if self.exported {
            if tokens.export.is_none() {
                tokens.export = Some(Token::from_content("export"));
            }
            tokens.export.as_mut().unwrap()
        } else {
            &mut tokens.r#type
        }
    }

    /// Returns a mutable reference to the last token for this statement,
    /// creating it if missing.
    pub fn mutate_last_token(&mut self) -> &mut Token {
        self.set_default_tokens();
        &mut self.tokens.as_mut().unwrap().end
    }

    fn set_default_tokens(&mut self) {
        if self.tokens.is_none() {
            self.tokens = Some(
                TypeFunctionStatementTokens {
                    r#type: Token::from_content("type"),
                    export: self.exported.then(|| Token::from_content("export")),
                    function_body: FunctionBodyTokens {
                        function: Token::from_content("function"),
                        opening_parenthese: Token::from_content("("),
                        closing_parenthese: Token::from_content(")"),
                        end: Token::from_content("end"),
                        parameter_commas: Vec::new(),
                        variable_arguments: None,
                        variable_arguments_colon: None,
                        return_type_colon: None,
                    },
                }
                .into(),
            );
        }
    }

    super::impl_token_fns!(
        target = [identifier]
        iter = [parameters, generic_parameters, tokens]
    );
}

/// Tokens associated with a type function statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeFunctionStatementTokens {
    pub r#type: Token,
    pub function_body: FunctionBodyTokens,
    pub export: Option<Token>,
}

impl TypeFunctionStatementTokens {
    super::impl_token_fns!(
        target = [r#type, function_body]
        iter = [export]
    );
}

impl std::ops::Deref for TypeFunctionStatementTokens {
    type Target = FunctionBodyTokens;

    fn deref(&self) -> &Self::Target {
        &self.function_body
    }
}

impl std::ops::DerefMut for TypeFunctionStatementTokens {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.function_body
    }
}
