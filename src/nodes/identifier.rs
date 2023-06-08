use crate::nodes::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier {
    name: String,
    token: Option<Token>,
}

impl Identifier {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
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
    pub fn get_name(&self) -> &String {
        &self.name
    }

    #[inline]
    pub fn mutate_name(&mut self) -> &mut String {
        &mut self.name
    }

    #[inline]
    pub fn set_name<IntoString: Into<String>>(&mut self, name: IntoString) {
        let name = name.into();
        if let Some(token) = &mut self.token {
            token.replace_with_content(name.clone());
        }
        self.name = name;
    }

    #[inline]
    pub fn into_name(self) -> String {
        self.name
    }

    pub fn clear_comments(&mut self) {
        if let Some(token) = &mut self.token {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(token) = &mut self.token {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(token) = &mut self.token {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(token) = &mut self.token {
            token.shift_token_line(amount);
        }
    }
}

impl<IntoString: Into<String>> From<IntoString> for Identifier {
    fn from(identifier: IntoString) -> Self {
        Self {
            name: identifier.into(),
            token: None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::nodes::Position;

    use super::*;

    #[test]
    fn set_name_replaces_the_token_content() {
        let token = Token::new_with_line(7, 10, 1);
        let mut identifier = Identifier::new("var").with_token(token);

        identifier.set_name("newVar");

        assert_eq!(
            identifier.get_token().unwrap(),
            &Token::from_position(Position::LineNumber {
                line_number: 1,
                content: "newVar".into(),
            })
        );
    }
}
