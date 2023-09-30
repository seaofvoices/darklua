use crate::nodes::{GenericParametersWithDefaults, Identifier, Token, Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDeclarationTokens {
    pub r#type: Token,
    pub equal: Token,
    pub export: Option<Token>,
}

impl TypeDeclarationTokens {
    pub fn clear_comments(&mut self) {
        self.r#type.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.r#type.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.r#type.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.r#type.shift_token_line(amount);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDeclarationStatement {
    name: Identifier,
    r#type: Type,
    exported: bool,
    generic_parameters: Option<GenericParametersWithDefaults>,
    tokens: Option<TypeDeclarationTokens>,
}

impl TypeDeclarationStatement {
    pub fn new(name: impl Into<Identifier>, r#type: impl Into<Type>) -> Self {
        Self {
            name: name.into(),
            r#type: r#type.into(),
            exported: false,
            generic_parameters: None,
            tokens: None,
        }
    }

    pub fn with_generic_parameters(
        mut self,
        generic_parameters: GenericParametersWithDefaults,
    ) -> Self {
        self.generic_parameters = Some(generic_parameters);
        self
    }

    #[inline]
    pub fn set_generic_parameters(&mut self, generic_parameters: GenericParametersWithDefaults) {
        self.generic_parameters = Some(generic_parameters);
    }

    #[inline]
    pub fn get_generic_parameters(&self) -> Option<&GenericParametersWithDefaults> {
        self.generic_parameters.as_ref()
    }

    pub fn export(mut self) -> Self {
        self.exported = true;
        self
    }

    #[inline]
    pub fn set_exported(&mut self) {
        self.exported = true;
    }

    #[inline]
    pub fn is_exported(&self) -> bool {
        self.exported
    }

    #[inline]
    pub fn get_type(&self) -> &Type {
        &self.r#type
    }

    #[inline]
    pub fn mutate_type(&mut self) -> &mut Type {
        &mut self.r#type
    }

    #[inline]
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    #[inline]
    pub fn mutate_name(&mut self) -> &mut Identifier {
        &mut self.name
    }

    pub fn with_tokens(mut self, tokens: TypeDeclarationTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: TypeDeclarationTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TypeDeclarationTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut TypeDeclarationTokens> {
        self.tokens.as_mut()
    }

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
    }
}
