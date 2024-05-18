use crate::nodes::{Arguments, Expression, Identifier, Prefix, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionCallTokens {
    pub colon: Option<Token>,
}

impl FunctionCallTokens {
    super::impl_token_fns!(iter = [colon]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionCall {
    prefix: Box<Prefix>,
    arguments: Arguments,
    method: Option<Identifier>,
    tokens: Option<FunctionCallTokens>,
}

impl FunctionCall {
    pub fn new(prefix: Prefix, arguments: Arguments, method: Option<Identifier>) -> Self {
        Self {
            prefix: Box::new(prefix),
            arguments,
            method,
            tokens: None,
        }
    }

    pub fn from_name<T: Into<Identifier>>(name: T) -> Self {
        Self {
            prefix: Box::new(name.into().into()),
            arguments: Arguments::default(),
            method: None,
            tokens: None,
        }
    }

    pub fn from_prefix<T: Into<Prefix>>(prefix: T) -> Self {
        Self {
            prefix: Box::new(prefix.into()),
            arguments: Arguments::default(),
            method: None,
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: FunctionCallTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: FunctionCallTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&FunctionCallTokens> {
        self.tokens.as_ref()
    }

    pub fn with_arguments<A: Into<Arguments>>(mut self, arguments: A) -> Self {
        self.arguments = arguments.into();
        self
    }

    pub fn with_argument<T: Into<Expression>>(mut self, argument: T) -> Self {
        self.arguments = self.arguments.with_argument(argument);
        self
    }

    pub fn with_method<IntoString: Into<Identifier>>(mut self, method: IntoString) -> Self {
        self.method.replace(method.into());
        self
    }

    #[inline]
    pub fn get_arguments(&self) -> &Arguments {
        &self.arguments
    }

    #[inline]
    pub fn get_method(&self) -> Option<&Identifier> {
        self.method.as_ref()
    }

    #[inline]
    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    #[inline]
    pub fn take_method(&mut self) -> Option<Identifier> {
        self.method.take()
    }

    #[inline]
    pub fn set_arguments(&mut self, arguments: Arguments) {
        self.arguments = arguments;
    }

    #[inline]
    pub fn set_method(&mut self, method: Identifier) {
        self.method.replace(method);
    }

    #[inline]
    pub fn mutate_arguments(&mut self) -> &mut Arguments {
        &mut self.arguments
    }

    #[inline]
    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }

    super::impl_token_fns!(iter = [tokens, method]);
}
