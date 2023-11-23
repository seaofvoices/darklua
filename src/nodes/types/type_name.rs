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

    pub fn clear_comments(&mut self) {
        self.type_name.clear_comments();
        if let Some(parameters) = &mut self.type_parameters {
            parameters.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.type_name.clear_comments();
        if let Some(parameters) = &mut self.type_parameters {
            parameters.clear_comments();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.type_name.replace_referenced_tokens(code);
        if let Some(parameters) = &mut self.type_parameters {
            parameters.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.type_name.shift_token_line(amount);
        if let Some(parameters) = &mut self.type_parameters {
            parameters.shift_token_line(amount);
        }
    }
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

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
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
        Self::Type(value.into())
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
    pub fn clear_comments(&mut self) {
        self.opening_list.clear_comments();
        self.closing_list.clear_comments();
        for comma in &mut self.commas {
            comma.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_list.clear_whitespaces();
        self.closing_list.clear_whitespaces();
        for comma in &mut self.commas {
            comma.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_list.replace_referenced_tokens(code);
        self.closing_list.replace_referenced_tokens(code);
        for comma in &mut self.commas {
            comma.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.opening_list.shift_token_line(amount);
        self.closing_list.shift_token_line(amount);
        for comma in &mut self.commas {
            comma.shift_token_line(amount);
        }
    }
}
