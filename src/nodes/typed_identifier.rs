use crate::nodes::{Identifier, Token, Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedIdentifier {
    name: Identifier,
    r#type: Option<Type>,
    token: Option<Token>,
}

impl TypedIdentifier {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Identifier::new(name.into()),
            r#type: None,
            token: None,
        }
    }

    pub fn with_type(mut self, type_value: impl Into<Type>) -> Self {
        self.r#type = Some(type_value.into());
        self
    }

    pub fn with_colon_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    #[inline]
    pub fn set_colon_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    #[inline]
    pub fn get_colon_token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    #[inline]
    pub fn get_identifier(&self) -> &Identifier {
        &self.name
    }

    #[inline]
    pub fn get_type(&self) -> Option<&Type> {
        self.r#type.as_ref()
    }

    #[inline]
    pub fn has_type(&self) -> bool {
        self.r#type.is_some()
    }

    #[inline]
    pub fn mutate_type(&mut self) -> Option<&mut Type> {
        self.r#type.as_mut()
    }

    pub fn remove_type(mut self) -> Option<Type> {
        self.token.take();
        self.r#type.take()
    }

    pub fn clear_comments(&mut self) {
        self.name.clear_comments();
        if let Some(token) = &mut self.token {
            token.clear_comments();
        }
        if let Some(r#type) = &mut self.r#type {
            r#type.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.name.clear_whitespaces();
        if let Some(token) = &mut self.token {
            token.clear_whitespaces();
        }
        if let Some(r#type) = &mut self.r#type {
            r#type.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.name.replace_referenced_tokens(code);
        if let Some(token) = &mut self.token {
            token.replace_referenced_tokens(code);
        }
        if let Some(r#type) = &mut self.r#type {
            r#type.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.name.shift_token_line(amount);
        if let Some(token) = &mut self.token {
            token.shift_token_line(amount);
        }
        if let Some(r#type) = &mut self.r#type {
            r#type.shift_token_line(amount);
        }
    }
}

impl<IntoIdentifier: Into<Identifier>> From<IntoIdentifier> for TypedIdentifier {
    fn from(name: IntoIdentifier) -> Self {
        Self {
            name: name.into(),
            r#type: None,
            token: None,
        }
    }
}

impl std::ops::Deref for TypedIdentifier {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl std::ops::DerefMut for TypedIdentifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.name
    }
}

#[cfg(test)]
mod test {
    use crate::nodes::Position;

    use super::*;

    #[test]
    fn set_name_replaces_the_token_content() {
        let token = Token::new_with_line(7, 10, 1);
        let mut typed_identifier = TypedIdentifier::from(Identifier::new("var").with_token(token));

        typed_identifier.set_name("newVar");

        assert_eq!(
            typed_identifier.get_identifier().get_token().unwrap(),
            &Token::from_position(Position::LineNumber {
                line_number: 1,
                content: "newVar".into(),
            })
        );
    }
}
