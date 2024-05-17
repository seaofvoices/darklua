use crate::nodes::{Identifier, Token};

use super::TypeName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeField {
    namespace: Identifier,
    name: TypeName,
    token: Option<Token>,
}

impl TypeField {
    pub fn new(namespace: impl Into<Identifier>, type_name: TypeName) -> Self {
        Self {
            namespace: namespace.into(),
            name: type_name,
            token: None,
        }
    }

    pub fn get_type_name(&self) -> &TypeName {
        &self.name
    }

    pub fn mutate_type_name(&mut self) -> &mut TypeName {
        &mut self.name
    }

    pub fn get_namespace(&self) -> &Identifier {
        &self.namespace
    }

    pub fn mutate_namespace(&mut self) -> &mut Identifier {
        &mut self.namespace
    }

    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    #[inline]
    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    #[inline]
    pub fn get_token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    super::impl_token_fns!(target = [namespace] iter = [token]);
}
