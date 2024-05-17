use std::iter::FromIterator;

use crate::nodes::Token;

use super::{Type, VariadicArgumentType};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypePack {
    types: Vec<Type>,
    variadic_type: Option<VariadicArgumentType>,
    tokens: Option<TypePackTokens>,
}

impl TypePack {
    pub fn with_type(mut self, r#type: impl Into<Type>) -> Self {
        self.types.push(r#type.into());
        self
    }

    pub fn push_type(&mut self, r#type: impl Into<Type>) {
        self.types.push(r#type.into());
    }

    pub fn with_variadic_type(mut self, variadic_type: impl Into<VariadicArgumentType>) -> Self {
        self.variadic_type = Some(variadic_type.into());
        self
    }

    pub fn set_variadic_type(&mut self, variadic_type: impl Into<VariadicArgumentType>) {
        self.variadic_type = Some(variadic_type.into());
    }

    #[inline]
    pub fn get_variadic_type(&self) -> Option<&VariadicArgumentType> {
        self.variadic_type.as_ref()
    }

    #[inline]
    pub fn has_variadic_type(&self) -> bool {
        self.variadic_type.is_some()
    }

    #[inline]
    pub fn mutate_variadic_type(&mut self) -> Option<&mut VariadicArgumentType> {
        self.variadic_type.as_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Type> {
        self.types.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Type> {
        self.types.iter_mut()
    }

    pub fn with_tokens(mut self, tokens: TypePackTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: TypePackTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TypePackTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut TypePackTokens> {
        self.tokens.as_mut()
    }

    super::impl_token_fns!(iter = [tokens]);
}

impl FromIterator<Type> for TypePack {
    fn from_iter<T: IntoIterator<Item = Type>>(iter: T) -> Self {
        Self {
            types: iter.into_iter().collect(),
            variadic_type: None,
            tokens: None,
        }
    }
}

impl IntoIterator for TypePack {
    type Item = Type;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.types.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut TypePack {
    type Item = &'a mut Type;
    type IntoIter = std::slice::IterMut<'a, Type>;

    fn into_iter(self) -> Self::IntoIter {
        self.types.iter_mut()
    }
}

impl<'a> IntoIterator for &'a TypePack {
    type Item = &'a Type;
    type IntoIter = std::slice::Iter<'a, Type>;

    fn into_iter(self) -> Self::IntoIter {
        self.types.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypePackTokens {
    pub left_parenthese: Token,
    pub right_parenthese: Token,
    pub commas: Vec<Token>,
}

impl TypePackTokens {
    super::impl_token_fns!(
        target = [left_parenthese, right_parenthese]
        iter = [commas]
    );
}
