use std::iter::FromIterator;

use crate::nodes::{Identifier, Token};

use super::{GenericTypePack, Type, TypePack, VariadicTypePack};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeName {
    type_name: Identifier,
    type_parameters: Option<TypeParameters>,
}

impl TypeName {
    pub fn new(type_name: impl Into<Identifier>) -> Self {
        Self {
            type_name: type_name.into(),
            type_parameters: None,
        }
    }

    pub fn with_type_parameters(mut self, type_parameters: TypeParameters) -> Self {
        self.type_parameters = Some(type_parameters);
        self
    }

    pub fn with_type_parameter(mut self, parameter: impl Into<TypeParameter>) -> Self {
        self.push_type_parameter(parameter.into());
        self
    }

    pub fn push_type_parameter(&mut self, parameter: impl Into<TypeParameter>) {
        if let Some(parameters) = &mut self.type_parameters {
            parameters.push_parameter(parameter.into());
        } else {
            self.type_parameters = Some(TypeParameters {
                parameters: vec![parameter.into()],
                tokens: None,
            })
        }
    }

    #[inline]
    pub fn get_type_name(&self) -> &Identifier {
        &self.type_name
    }

    #[inline]
    pub fn mutate_type_name(&mut self) -> &mut Identifier {
        &mut self.type_name
    }

    #[inline]
    pub fn get_type_parameters(&self) -> Option<&TypeParameters> {
        self.type_parameters.as_ref()
    }

    #[inline]
    pub fn has_type_parameters(&self) -> bool {
        self.type_parameters.is_some()
    }

    #[inline]
    pub fn mutate_type_parameters(&mut self) -> Option<&mut TypeParameters> {
        self.type_parameters.as_mut()
    }

    super::impl_token_fns!(target = [type_name] iter = [type_parameters]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParameters {
    parameters: Vec<TypeParameter>,
    tokens: Option<TypeParametersTokens>,
}

impl TypeParameters {
    pub fn new(parameter: impl Into<TypeParameter>) -> Self {
        Self {
            parameters: vec![parameter.into()],
            tokens: None,
        }
    }

    pub fn with_parameter(mut self, parameter: impl Into<TypeParameter>) -> Self {
        self.parameters.push(parameter.into());
        self
    }

    pub fn push_parameter(&mut self, parameter: impl Into<TypeParameter>) {
        self.parameters.push(parameter.into());
    }

    pub fn with_tokens(mut self, tokens: TypeParametersTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &TypeParameter> {
        self.parameters.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TypeParameter> {
        self.parameters.iter_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: TypeParametersTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TypeParametersTokens> {
        self.tokens.as_ref()
    }

    super::impl_token_fns!(iter = [tokens]);
}

impl FromIterator<TypeParameter> for TypeParameters {
    fn from_iter<T: IntoIterator<Item = TypeParameter>>(iter: T) -> Self {
        Self {
            parameters: iter.into_iter().collect(),
            tokens: None,
        }
    }
}

impl IntoIterator for TypeParameters {
    type Item = TypeParameter;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.parameters.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut TypeParameters {
    type Item = &'a mut TypeParameter;
    type IntoIter = std::slice::IterMut<'a, TypeParameter>;

    fn into_iter(self) -> Self::IntoIter {
        self.parameters.iter_mut()
    }
}

impl<'a> IntoIterator for &'a TypeParameters {
    type Item = &'a TypeParameter;
    type IntoIter = std::slice::Iter<'a, TypeParameter>;

    fn into_iter(self) -> Self::IntoIter {
        self.parameters.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeParameter {
    Type(Type),
    TypePack(TypePack),
    VariadicTypePack(VariadicTypePack),
    GenericTypePack(GenericTypePack),
}

impl<T: Into<Type>> From<T> for TypeParameter {
    fn from(value: T) -> Self {
        match value.into() {
            Type::Parenthese(parenthese) => {
                Self::TypePack(TypePack::default().with_type(parenthese.into_inner_type()))
            }
            other => Self::Type(other),
        }
    }
}

impl From<TypePack> for TypeParameter {
    fn from(value: TypePack) -> Self {
        Self::TypePack(value)
    }
}

impl From<VariadicTypePack> for TypeParameter {
    fn from(value: VariadicTypePack) -> Self {
        Self::VariadicTypePack(value)
    }
}

impl From<GenericTypePack> for TypeParameter {
    fn from(value: GenericTypePack) -> Self {
        Self::GenericTypePack(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParametersTokens {
    pub opening_list: Token,
    pub closing_list: Token,
    pub commas: Vec<Token>,
}

impl TypeParametersTokens {
    super::impl_token_fns!(
        target = [opening_list, closing_list]
        iter = [commas]
    );
}
