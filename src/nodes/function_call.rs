use crate::nodes::{
    Arguments, Expression, Identifier, Prefix, Token, Type, TypeInstantiationTokens,
};

/// Tokens associated with a function call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionCallTokens {
    pub colon: Option<Token>,
    pub type_instantiation_tokens: Option<TypeInstantiationTokens>,
}

impl FunctionCallTokens {
    super::impl_token_fns!(iter = [colon, type_instantiation_tokens]);
}

/// Represents a function call expression (e.g., `func()`, `obj:method()`, `a.b.c()`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionCall {
    prefix: Box<Prefix>,
    arguments: Arguments,
    method: Option<Method>,
    tokens: Option<FunctionCallTokens>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Method {
    name: Identifier,
    types: Option<Vec<Type>>,
}

impl Method {
    super::impl_token_fns!(target = [name]);
}

impl FunctionCall {
    /// Creates a new function call with the given prefix, arguments, and optional method.
    pub fn new(prefix: Prefix, arguments: Arguments, method: Option<Identifier>) -> Self {
        Self {
            prefix: Box::new(prefix),
            arguments,
            method: method.map(|name| Method { name, types: None }),
            tokens: None,
        }
    }

    /// Creates a new function call with the given name.
    pub fn from_name<T: Into<Identifier>>(name: T) -> Self {
        Self {
            prefix: Box::new(name.into().into()),
            arguments: Arguments::default(),
            method: None,
            tokens: None,
        }
    }

    /// Creates a new function call with the given prefix.
    pub fn from_prefix<T: Into<Prefix>>(prefix: T) -> Self {
        Self {
            prefix: Box::new(prefix.into()),
            arguments: Arguments::default(),
            method: None,
            tokens: None,
        }
    }

    /// Sets the tokens for this function call.
    pub fn with_tokens(mut self, tokens: FunctionCallTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Sets the tokens for this function call.
    #[inline]
    pub fn set_tokens(&mut self, tokens: FunctionCallTokens) {
        self.tokens = Some(tokens);
    }

    /// Returns the tokens for this function call, if any.
    #[inline]
    pub fn get_tokens(&self) -> Option<&FunctionCallTokens> {
        self.tokens.as_ref()
    }

    /// Sets the arguments for this function call.
    pub fn with_arguments<A: Into<Arguments>>(mut self, arguments: A) -> Self {
        self.arguments = arguments.into();
        self
    }

    /// Adds an argument to this function call.
    pub fn with_argument<T: Into<Expression>>(mut self, argument: T) -> Self {
        self.arguments = self.arguments.with_argument(argument);
        self
    }

    /// Sets the method name for this function call (for method calls like `obj:method()`).
    pub fn with_method(mut self, method: impl Into<Identifier>) -> Self {
        self.set_method(method.into());
        self
    }

    /// Sets the method with a specific type instantiation.
    pub fn with_type_instantiation_method(
        mut self,
        method: impl Into<Identifier>,
        types: Vec<Type>,
    ) -> Self {
        self.set_type_instantiation_method(method.into(), types);
        self
    }

    /// Sets the method with a specific type instantiation.
    pub fn set_type_instantiation_method(
        &mut self,
        method: impl Into<Identifier>,
        types: Vec<Type>,
    ) {
        self.method = Some(Method {
            name: method.into(),
            types: Some(types),
        });
    }

    /// Removes the type instantiation from the method, if any, and returns true if it was present.
    pub fn remove_type_instantiation_from_method(&mut self) -> bool {
        self.method
            .as_mut()
            .and_then(|method| method.types.take())
            .is_some()
    }

    /// Returns the arguments of this function call.
    #[inline]
    pub fn get_arguments(&self) -> &Arguments {
        &self.arguments
    }

    /// Returns the method name, if this is a method call.
    #[inline]
    pub fn get_method(&self) -> Option<&Identifier> {
        self.method.as_ref().map(|method| &method.name)
    }

    /// Returns an iterator over the type instantiations of the method.
    pub fn get_method_type_instantiation(&self) -> impl Iterator<Item = &Type> {
        self.method
            .iter()
            .flat_map(|method| method.types.iter().flatten())
    }

    /// Returns whether this call has a method with a type instantiation.
    pub fn has_method_type_instantiation(&self) -> bool {
        self.method
            .as_ref()
            .map(|method| method.types.is_some())
            .unwrap_or_default()
    }

    /// Returns if this call uses a method.
    #[inline]
    pub fn has_method(&self) -> bool {
        self.method.is_some()
    }

    /// Returns the prefix (what is being called) of this function call.
    #[inline]
    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    /// Removes and returns the method name, if any.
    #[inline]
    pub fn take_method(&mut self) -> Option<Identifier> {
        let method = self.method.take();
        if let Some(tokens) = self.tokens.as_mut() {
            tokens.colon = None;
        }
        method.map(|method| method.name)
    }

    /// Sets the arguments for this function call.
    #[inline]
    pub fn set_arguments(&mut self, arguments: Arguments) {
        self.arguments = arguments;
    }

    /// Sets the method name for this function call.
    #[inline]
    pub fn set_method(&mut self, method: Identifier) {
        if let Some(current_method) = &mut self.method {
            current_method.name = method;
        } else {
            self.method = Some(Method {
                name: method,
                types: None,
            });
        }
    }

    /// Returns a mutable reference to the arguments.
    #[inline]
    pub fn mutate_arguments(&mut self) -> &mut Arguments {
        &mut self.arguments
    }

    /// Returns a mutable reference to the prefix.
    #[inline]
    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }

    /// Returns a mutable reference to the first token of this function call,
    /// creating it if missing.
    pub fn mutate_first_token(&mut self) -> &mut Token {
        self.prefix.mutate_first_token()
    }

    /// Returns a mutable reference to the last token of this function call,
    /// creating it if missing.
    pub fn mutate_last_token(&mut self) -> &mut Token {
        self.arguments.mutate_last_token()
    }

    super::impl_token_fns!(iter = [tokens, method]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{nodes::Statement, Parser};

    fn parse_call(code: &str) -> FunctionCall {
        let parser = Parser::default().preserve_tokens();
        let block = parser.parse(code).expect("code should parse");
        if let Some(Statement::Call(call)) = block.first_statement() {
            return call.clone();
        }
        panic!("failed to parse call from: {}", code);
    }

    #[test]
    fn test_take_method_removes_colon_token() {
        let mut call = parse_call("obj:method()");

        assert!(call.get_tokens().unwrap().colon.is_some());
        call.take_method();
        assert!(call.get_tokens().unwrap().colon.is_none());
    }
}
