use std::iter;

use crate::nodes::Token;

use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntersectionType {
    types: Vec<Type>,
    leading_operator: bool,
    tokens: Option<IntersectionTypeTokens>,
}

impl IntersectionType {
    pub fn new(left_type: impl Into<Type>, right_type: impl Into<Type>) -> Self {
        Self {
            types: vec![left_type.into(), right_type.into()],
            leading_operator: false,
            tokens: None,
        }
    }

    pub fn with_type(mut self, r#type: impl Into<Type>) -> Self {
        self.types.push(r#type.into());
        self
    }

    pub fn with_tokens(mut self, tokens: IntersectionTypeTokens) -> Self {
        self.set_tokens(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: IntersectionTypeTokens) {
        if tokens.leading_token.is_some() {
            self.leading_operator = true;
        }
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_token(&self) -> Option<&IntersectionTypeTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.types.len()
    }

    #[inline]
    pub fn iter_types(&self) -> impl Iterator<Item = &Type> {
        self.types.iter()
    }

    #[inline]
    pub fn iter_mut_types(&mut self) -> impl Iterator<Item = &mut Type> {
        self.types.iter_mut()
    }

    #[inline]
    pub fn first_type(&self) -> &Type {
        self.types.first().unwrap()
    }

    #[inline]
    pub fn last_type(&self) -> &Type {
        self.types.last().unwrap()
    }

    pub fn has_leading_token(&self) -> bool {
        self.leading_operator
            || self.types.len() < 2
            || self
                .tokens
                .as_ref()
                .map(|tokens| tokens.leading_token.is_some())
                .unwrap_or_default()
    }

    pub fn with_leading_token(mut self) -> Self {
        self.put_leading_token();
        self
    }

    pub fn put_leading_token(&mut self) {
        self.leading_operator = true;
    }

    pub fn remove_leading_token(&mut self) {
        self.leading_operator = false;
        if let Some(tokens) = &mut self.tokens {
            tokens.leading_token.take();
        }
    }

    pub fn intermediate_needs_parentheses(r#type: &Type) -> bool {
        matches!(
            r#type,
            Type::Optional(_) | Type::Union(_) | Type::Function(_) | Type::Intersection(_)
        )
    }

    pub fn last_needs_parentheses(r#type: &Type) -> bool {
        matches!(
            r#type,
            Type::Optional(_) | Type::Union(_) | Type::Function(_) | Type::Intersection(_)
        )
    }

    super::impl_token_fns!(iter = [tokens]);
}

impl From<Vec<Type>> for IntersectionType {
    fn from(types: Vec<Type>) -> Self {
        assert!(!types.is_empty(), "union types cannot be empty");
        Self {
            types,
            leading_operator: false,
            tokens: None,
        }
    }
}

impl iter::FromIterator<Type> for IntersectionType {
    fn from_iter<I: IntoIterator<Item = Type>>(iter: I) -> Self {
        Self {
            types: iter.into_iter().collect(),
            leading_operator: false,
            tokens: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntersectionTypeTokens {
    pub leading_token: Option<Token>,
    pub separators: Vec<Token>,
}

impl IntersectionTypeTokens {
    super::impl_token_fns!(iter = [leading_token, separators]);
}
