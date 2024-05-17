use std::marker::PhantomData;

use crate::nodes::{Identifier, Token, TypePack, VariadicTypePack};

use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericTypePack {
    // name ...
    name: Identifier,
    token: Option<Token>,
}

impl GenericTypePack {
    pub fn new(name: impl Into<Identifier>) -> Self {
        Self {
            name: name.into(),
            token: None,
        }
    }

    #[inline]
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    #[inline]
    pub fn mutate_name(&mut self) -> &mut Identifier {
        &mut self.name
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

    super::impl_token_fns!(
        target = [name]
        iter = [token]
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericParameters {
    // generic type list
    type_variables: Vec<Identifier>,
    generic_type_packs: Vec<GenericTypePack>,
    tokens: Option<GenericParametersTokens>,
}

impl GenericParameters {
    pub fn from_type_variable(name: impl Into<Identifier>) -> Self {
        Self {
            type_variables: vec![name.into()],
            generic_type_packs: Vec::new(),
            tokens: None,
        }
    }
    pub fn from_generic_type_pack(generic_type_pack: impl Into<GenericTypePack>) -> Self {
        Self {
            type_variables: Vec::new(),
            generic_type_packs: vec![generic_type_pack.into()],
            tokens: None,
        }
    }

    pub fn with_type_variable(mut self, type_variable: impl Into<Identifier>) -> Self {
        self.type_variables.push(type_variable.into());
        self
    }

    pub fn push_type_variable(&mut self, type_variable: impl Into<Identifier>) {
        self.type_variables.push(type_variable.into());
    }

    pub fn push_generic_type_pack(&mut self, generic_pack: GenericTypePack) {
        self.generic_type_packs.push(generic_pack);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.type_variables.len() + self.generic_type_packs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.type_variables.is_empty() && self.generic_type_packs.is_empty()
    }

    #[inline]
    pub fn type_variables_len(&self) -> usize {
        self.type_variables.len()
    }

    #[inline]
    pub fn generic_type_packs_len(&self) -> usize {
        self.generic_type_packs.len()
    }

    pub fn iter_type_variable(&self) -> impl Iterator<Item = &Identifier> {
        self.type_variables.iter()
    }

    pub fn iter_mut_type_variable(&mut self) -> impl Iterator<Item = &mut Identifier> {
        self.type_variables.iter_mut()
    }

    pub fn iter_generic_type_pack(&self) -> impl Iterator<Item = &GenericTypePack> {
        self.generic_type_packs.iter()
    }

    pub fn iter_mut_generic_type_pack(&mut self) -> impl Iterator<Item = &mut GenericTypePack> {
        self.generic_type_packs.iter_mut()
    }

    pub fn with_tokens(mut self, tokens: GenericParametersTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: GenericParametersTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&GenericParametersTokens> {
        self.tokens.as_ref()
    }

    super::impl_token_fns!(iter = [type_variables, generic_type_packs, tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericParametersTokens {
    pub opening_list: Token,
    pub closing_list: Token,
    pub commas: Vec<Token>,
}

impl GenericParametersTokens {
    super::impl_token_fns!(
        target = [opening_list, closing_list]
        iter = [commas]
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericTypePackDefault {
    TypePack(TypePack),
    VariadicTypePack(VariadicTypePack),
    GenericTypePack(GenericTypePack),
}

impl From<TypePack> for GenericTypePackDefault {
    fn from(type_pack: TypePack) -> Self {
        Self::TypePack(type_pack)
    }
}

impl From<VariadicTypePack> for GenericTypePackDefault {
    fn from(variadic_type_pack: VariadicTypePack) -> Self {
        Self::VariadicTypePack(variadic_type_pack)
    }
}

impl From<GenericTypePack> for GenericTypePackDefault {
    fn from(generic_type_pack: GenericTypePack) -> Self {
        Self::GenericTypePack(generic_type_pack)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericTypePackWithDefault {
    generic_type_pack: GenericTypePack,
    default: GenericTypePackDefault,
    // equal sign token
    token: Option<Token>,
}

impl GenericTypePackWithDefault {
    pub fn new(
        generic_type_pack: impl Into<GenericTypePack>,
        default: impl Into<GenericTypePackDefault>,
    ) -> Self {
        Self {
            generic_type_pack: generic_type_pack.into(),
            default: default.into(),
            token: None,
        }
    }

    #[inline]
    pub fn get_generic_type_pack(&self) -> &GenericTypePack {
        &self.generic_type_pack
    }

    #[inline]
    pub fn mutate_generic_type_pack(&mut self) -> &mut GenericTypePack {
        &mut self.generic_type_pack
    }

    #[inline]
    pub fn get_default_type(&self) -> &GenericTypePackDefault {
        &self.default
    }

    #[inline]
    pub fn mutate_default_type(&mut self) -> &mut GenericTypePackDefault {
        &mut self.default
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

    super::impl_token_fns!(iter = [token]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeVariableWithDefault {
    variable: Identifier,
    default: Type,
    // equal sign token
    token: Option<Token>,
}

impl TypeVariableWithDefault {
    pub fn new(identifier: impl Into<Identifier>, r#type: impl Into<Type>) -> Self {
        Self {
            variable: identifier.into(),
            default: r#type.into(),
            token: None,
        }
    }

    #[inline]
    pub fn get_type_variable(&self) -> &Identifier {
        &self.variable
    }

    #[inline]
    pub fn mutate_type_variable(&mut self) -> &mut Identifier {
        &mut self.variable
    }

    #[inline]
    pub fn get_default_type(&self) -> &Type {
        &self.default
    }

    #[inline]
    pub fn mutate_default_type(&mut self) -> &mut Type {
        &mut self.default
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

    super::impl_token_fns!(
        target = [variable]
        iter = [token]
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericParametersWithDefaults {
    type_variables: Vec<Identifier>,
    middle: GenericParametersWithDefaultsMiddle,
    generic_type_packs_with_default: Vec<GenericTypePackWithDefault>,
    tokens: Option<GenericParametersTokens>,
}

impl GenericParametersWithDefaults {
    pub fn from_type_variable(identifier: impl Into<Identifier>) -> Self {
        Self {
            type_variables: vec![identifier.into()],
            middle: GenericParametersWithDefaultsMiddle::Empty,
            generic_type_packs_with_default: Vec::new(),
            tokens: None,
        }
    }

    pub fn from_type_variable_with_default(
        type_variable_with_default: TypeVariableWithDefault,
    ) -> Self {
        Self {
            type_variables: Vec::new(),
            middle: GenericParametersWithDefaultsMiddle::TypeVariableDefaults(vec![
                type_variable_with_default,
            ]),
            generic_type_packs_with_default: Vec::new(),
            tokens: None,
        }
    }

    pub fn from_generic_type_pack(generic_type_pack: GenericTypePack) -> Self {
        Self {
            type_variables: Vec::new(),
            middle: GenericParametersWithDefaultsMiddle::GenericTypePacks(vec![generic_type_pack]),
            generic_type_packs_with_default: Vec::new(),
            tokens: None,
        }
    }

    pub fn from_generic_type_pack_with_default(
        generic_type_pack_with_default: GenericTypePackWithDefault,
    ) -> Self {
        Self {
            type_variables: Vec::new(),
            middle: GenericParametersWithDefaultsMiddle::Empty,
            generic_type_packs_with_default: vec![generic_type_pack_with_default],
            tokens: None,
        }
    }

    pub fn with_type_variable(mut self, identifier: impl Into<Identifier>) -> Self {
        self.type_variables.push(identifier.into());
        self
    }

    pub fn push_type_variable(&mut self, type_variable: impl Into<Identifier>) {
        self.type_variables.push(type_variable.into());
    }

    pub fn with_type_variable_with_default(
        mut self,
        type_variable_with_default: TypeVariableWithDefault,
    ) -> Option<Self> {
        if self.push_type_variable_with_default(type_variable_with_default) {
            Some(self)
        } else {
            None
        }
    }

    pub fn push_type_variable_with_default(
        &mut self,
        type_variable_with_default: TypeVariableWithDefault,
    ) -> bool {
        match &mut self.middle {
            GenericParametersWithDefaultsMiddle::Empty => {
                self.middle = GenericParametersWithDefaultsMiddle::TypeVariableDefaults(vec![
                    type_variable_with_default,
                ]);
                true
            }
            GenericParametersWithDefaultsMiddle::TypeVariableDefaults(types) => {
                types.push(type_variable_with_default);
                true
            }
            GenericParametersWithDefaultsMiddle::GenericTypePacks(_) => false,
        }
    }

    pub fn with_generic_type_pack(mut self, generic_type_pack: GenericTypePack) -> Option<Self> {
        if self.push_generic_type_pack(generic_type_pack) {
            Some(self)
        } else {
            None
        }
    }

    pub fn push_generic_type_pack(&mut self, generic_type_pack: GenericTypePack) -> bool {
        match &mut self.middle {
            GenericParametersWithDefaultsMiddle::Empty => {
                self.middle =
                    GenericParametersWithDefaultsMiddle::GenericTypePacks(vec![generic_type_pack]);
                true
            }
            GenericParametersWithDefaultsMiddle::GenericTypePacks(type_packs) => {
                type_packs.push(generic_type_pack);
                true
            }
            GenericParametersWithDefaultsMiddle::TypeVariableDefaults(_) => false,
        }
    }

    pub fn with_generic_type_pack_with_default(
        mut self,
        generic_type_pack_with_default: GenericTypePackWithDefault,
    ) -> Self {
        self.generic_type_packs_with_default
            .push(generic_type_pack_with_default);
        self
    }

    pub fn push_generic_type_pack_with_default(
        &mut self,
        generic_type_pack_with_default: GenericTypePackWithDefault,
    ) {
        self.generic_type_packs_with_default
            .push(generic_type_pack_with_default);
    }

    pub fn with_tokens(mut self, tokens: GenericParametersTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: GenericParametersTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&GenericParametersTokens> {
        self.tokens.as_ref()
    }

    pub fn iter(&self) -> impl Iterator<Item = GenericParameterRef<'_>> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = GenericParameterMutRef<'_>> {
        self.into_iter()
    }

    pub fn len(&self) -> usize {
        self.type_variables.len() + self.middle.len() + self.generic_type_packs_with_default.len()
    }

    pub fn is_empty(&self) -> bool {
        self.type_variables.is_empty()
            && self.middle.is_empty()
            && self.generic_type_packs_with_default.is_empty()
    }

    super::impl_token_fns!(iter = [tokens]);
}

pub enum GenericParameter {
    TypeVariable(Identifier),
    TypeVariableWithDefault(TypeVariableWithDefault),
    GenericTypePack(GenericTypePack),
    GenericTypePackWithDefault(GenericTypePackWithDefault),
}

impl From<Identifier> for GenericParameter {
    fn from(value: Identifier) -> Self {
        Self::TypeVariable(value)
    }
}

impl From<TypeVariableWithDefault> for GenericParameter {
    fn from(value: TypeVariableWithDefault) -> Self {
        Self::TypeVariableWithDefault(value)
    }
}

impl From<GenericTypePack> for GenericParameter {
    fn from(value: GenericTypePack) -> Self {
        Self::GenericTypePack(value)
    }
}

impl From<GenericTypePackWithDefault> for GenericParameter {
    fn from(value: GenericTypePackWithDefault) -> Self {
        Self::GenericTypePackWithDefault(value)
    }
}

pub enum GenericParameterRef<'a> {
    TypeVariable(&'a Identifier),
    TypeVariableWithDefault(&'a TypeVariableWithDefault),
    GenericTypePack(&'a GenericTypePack),
    GenericTypePackWithDefault(&'a GenericTypePackWithDefault),
}

impl<'a> From<&'a Identifier> for GenericParameterRef<'a> {
    fn from(value: &'a Identifier) -> Self {
        Self::TypeVariable(value)
    }
}

impl<'a> From<&'a TypeVariableWithDefault> for GenericParameterRef<'a> {
    fn from(value: &'a TypeVariableWithDefault) -> Self {
        Self::TypeVariableWithDefault(value)
    }
}

impl<'a> From<&'a GenericTypePack> for GenericParameterRef<'a> {
    fn from(value: &'a GenericTypePack) -> Self {
        Self::GenericTypePack(value)
    }
}

impl<'a> From<&'a GenericTypePackWithDefault> for GenericParameterRef<'a> {
    fn from(value: &'a GenericTypePackWithDefault) -> Self {
        Self::GenericTypePackWithDefault(value)
    }
}

pub enum GenericParameterMutRef<'a> {
    TypeVariable(&'a mut Identifier),
    TypeVariableWithDefault(&'a mut TypeVariableWithDefault),
    GenericTypePack(&'a mut GenericTypePack),
    GenericTypePackWithDefault(&'a mut GenericTypePackWithDefault),
}

impl<'a> From<&'a mut Identifier> for GenericParameterMutRef<'a> {
    fn from(value: &'a mut Identifier) -> Self {
        Self::TypeVariable(value)
    }
}

impl<'a> From<&'a mut TypeVariableWithDefault> for GenericParameterMutRef<'a> {
    fn from(value: &'a mut TypeVariableWithDefault) -> Self {
        Self::TypeVariableWithDefault(value)
    }
}

impl<'a> From<&'a mut GenericTypePack> for GenericParameterMutRef<'a> {
    fn from(value: &'a mut GenericTypePack) -> Self {
        Self::GenericTypePack(value)
    }
}

impl<'a> From<&'a mut GenericTypePackWithDefault> for GenericParameterMutRef<'a> {
    fn from(value: &'a mut GenericTypePackWithDefault) -> Self {
        Self::GenericTypePackWithDefault(value)
    }
}

pub struct GenericParameterIteratorGeneric<Item, A, B, C, D, IterA, IterB, IterC, IterD>
where
    IterA: Iterator<Item = A>,
    IterB: Iterator<Item = B>,
    IterC: Iterator<Item = C>,
    IterD: Iterator<Item = D>,
{
    type_variables: IterA,
    generic_type_packs: Option<IterB>,
    type_variables_with_default: Option<IterC>,
    generic_type_packs_with_default: IterD,
    _phantom: PhantomData<Item>,
}

impl<Item, A, B, C, D, IterA, IterB, IterC, IterD>
    GenericParameterIteratorGeneric<Item, A, B, C, D, IterA, IterB, IterC, IterD>
where
    IterA: Iterator<Item = A>,
    IterB: Iterator<Item = B>,
    IterC: Iterator<Item = C>,
    IterD: Iterator<Item = D>,
{
    fn new(
        type_variables: impl IntoIterator<Item = A, IntoIter = IterA>,
        generic_type_packs: Option<impl IntoIterator<Item = B, IntoIter = IterB>>,
        type_variables_with_default: Option<impl IntoIterator<Item = C, IntoIter = IterC>>,
        generic_type_packs_with_default: impl IntoIterator<Item = D, IntoIter = IterD>,
    ) -> Self {
        Self {
            type_variables: type_variables.into_iter(),
            generic_type_packs: generic_type_packs.map(IntoIterator::into_iter),
            type_variables_with_default: type_variables_with_default.map(IntoIterator::into_iter),
            generic_type_packs_with_default: generic_type_packs_with_default.into_iter(),
            _phantom: PhantomData,
        }
    }
}

pub type GenericParameterIterator = GenericParameterIteratorGeneric<
    GenericParameter,
    Identifier,
    GenericTypePack,
    TypeVariableWithDefault,
    GenericTypePackWithDefault,
    std::vec::IntoIter<Identifier>,
    std::vec::IntoIter<GenericTypePack>,
    std::vec::IntoIter<TypeVariableWithDefault>,
    std::vec::IntoIter<GenericTypePackWithDefault>,
>;

pub type GenericParameterRefIterator<'a> = GenericParameterIteratorGeneric<
    GenericParameterRef<'a>,
    &'a Identifier,
    &'a GenericTypePack,
    &'a TypeVariableWithDefault,
    &'a GenericTypePackWithDefault,
    std::slice::Iter<'a, Identifier>,
    std::slice::Iter<'a, GenericTypePack>,
    std::slice::Iter<'a, TypeVariableWithDefault>,
    std::slice::Iter<'a, GenericTypePackWithDefault>,
>;

pub type GenericParameterMutRefIterator<'a> = GenericParameterIteratorGeneric<
    GenericParameterMutRef<'a>,
    &'a mut Identifier,
    &'a mut GenericTypePack,
    &'a mut TypeVariableWithDefault,
    &'a mut GenericTypePackWithDefault,
    std::slice::IterMut<'a, Identifier>,
    std::slice::IterMut<'a, GenericTypePack>,
    std::slice::IterMut<'a, TypeVariableWithDefault>,
    std::slice::IterMut<'a, GenericTypePackWithDefault>,
>;

impl<IterItem, A, B, C, D, IterA, IterB, IterC, IterD> Iterator
    for GenericParameterIteratorGeneric<IterItem, A, B, C, D, IterA, IterB, IterC, IterD>
where
    A: Into<IterItem>,
    B: Into<IterItem>,
    C: Into<IterItem>,
    D: Into<IterItem>,
    IterA: Iterator<Item = A>,
    IterB: Iterator<Item = B>,
    IterC: Iterator<Item = C>,
    IterD: Iterator<Item = D>,
{
    type Item = IterItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.type_variables
            .next()
            .map(Into::into)
            .or_else(|| {
                self.generic_type_packs
                    .as_mut()
                    .and_then(|generic_type_packs| generic_type_packs.next().map(Into::into))
            })
            .or_else(|| {
                self.type_variables_with_default
                    .as_mut()
                    .and_then(|type_variables_with_default| {
                        type_variables_with_default.next().map(Into::into)
                    })
            })
            .or_else(|| self.generic_type_packs_with_default.next().map(Into::into))
    }
}

impl IntoIterator for GenericParametersWithDefaults {
    type Item = GenericParameter;
    type IntoIter = GenericParameterIterator;

    fn into_iter(self) -> Self::IntoIter {
        let (generic_type_packs, type_variables_with_default) = match self.middle {
            GenericParametersWithDefaultsMiddle::Empty => (None, None),
            GenericParametersWithDefaultsMiddle::GenericTypePacks(generic_type_packs) => {
                (Some(generic_type_packs), None)
            }
            GenericParametersWithDefaultsMiddle::TypeVariableDefaults(
                type_variables_with_default,
            ) => (None, Some(type_variables_with_default)),
        };
        GenericParameterIterator::new(
            self.type_variables,
            generic_type_packs,
            type_variables_with_default,
            self.generic_type_packs_with_default,
        )
    }
}

impl<'a> IntoIterator for &'a mut GenericParametersWithDefaults {
    type Item = GenericParameterMutRef<'a>;
    type IntoIter = GenericParameterMutRefIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let (generic_type_packs, type_variables_with_default) = match &mut self.middle {
            GenericParametersWithDefaultsMiddle::Empty => (None, None),
            GenericParametersWithDefaultsMiddle::GenericTypePacks(generic_type_packs) => {
                (Some(generic_type_packs), None)
            }
            GenericParametersWithDefaultsMiddle::TypeVariableDefaults(
                type_variables_with_default,
            ) => (None, Some(type_variables_with_default)),
        };
        GenericParameterMutRefIterator::new(
            &mut self.type_variables,
            generic_type_packs,
            type_variables_with_default,
            &mut self.generic_type_packs_with_default,
        )
    }
}

impl<'a> IntoIterator for &'a GenericParametersWithDefaults {
    type Item = GenericParameterRef<'a>;
    type IntoIter = GenericParameterRefIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let (generic_type_packs, type_variables_with_default) = match &self.middle {
            GenericParametersWithDefaultsMiddle::Empty => (None, None),
            GenericParametersWithDefaultsMiddle::GenericTypePacks(generic_type_packs) => {
                (Some(generic_type_packs), None)
            }
            GenericParametersWithDefaultsMiddle::TypeVariableDefaults(
                type_variables_with_default,
            ) => (None, Some(type_variables_with_default)),
        };
        GenericParameterRefIterator::new(
            &self.type_variables,
            generic_type_packs,
            type_variables_with_default,
            &self.generic_type_packs_with_default,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GenericParametersWithDefaultsMiddle {
    Empty,
    GenericTypePacks(Vec<GenericTypePack>),
    TypeVariableDefaults(Vec<TypeVariableWithDefault>),
}

impl GenericParametersWithDefaultsMiddle {
    fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::GenericTypePacks(generic_type_packs) => generic_type_packs.len(),
            Self::TypeVariableDefaults(type_variables) => type_variables.len(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::GenericTypePacks(generic_type_packs) => generic_type_packs.is_empty(),
            Self::TypeVariableDefaults(type_variables) => type_variables.is_empty(),
        }
    }
}
