use crate::nodes::{Identifier, Prefix, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldExpression {
    prefix: Prefix,
    field: Identifier,
    token: Option<Token>,
}

impl FieldExpression {
    pub fn new<IntoPrefix: Into<Prefix>, IntoIdentifier: Into<Identifier>>(
        prefix: IntoPrefix,
        field: IntoIdentifier,
    ) -> Self {
        Self {
            prefix: prefix.into(),
            field: field.into(),
            token: None,
        }
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

    #[inline]
    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    #[inline]
    pub fn get_field(&self) -> &Identifier {
        &self.field
    }

    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }

    super::impl_token_fns!(
        target = [field]
        iter = [token]
    );
}
