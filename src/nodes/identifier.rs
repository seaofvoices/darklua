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
    pub fn get_name(&self) -> &String {
        &self.name
    }

    #[inline]
    pub fn mutate_name(&mut self) -> &mut String {
        &mut self.name
    }

    #[inline]
    pub fn set_name<IntoString: Into<String>>(&mut self, name: IntoString) {
        self.name = name.into();
    }

    #[inline]
    pub fn into_name(self) -> String {
        self.name
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
