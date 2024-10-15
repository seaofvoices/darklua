use crate::nodes::{Identifier, Token};

use super::{GenericParameters, GenericTypePack, Type, TypePack, VariadicTypePack};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionArgumentType {
    argument_type: Type,
    name: Option<Identifier>,
    token: Option<Token>,
}

impl<T: Into<Type>> From<T> for FunctionArgumentType {
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}

impl FunctionArgumentType {
    pub fn new(argument_type: impl Into<Type>) -> Self {
        Self {
            argument_type: argument_type.into(),
            name: None,
            token: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<Identifier>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn set_name(&mut self, name: impl Into<Identifier>) {
        self.name = Some(name.into());
    }

    #[inline]
    pub fn get_type(&self) -> &Type {
        &self.argument_type
    }

    #[inline]
    pub fn mutate_type(&mut self) -> &mut Type {
        &mut self.argument_type
    }

    #[inline]
    pub fn get_name(&self) -> Option<&Identifier> {
        self.name.as_ref()
    }

    #[inline]
    pub fn mutate_name(&mut self) -> Option<&mut Identifier> {
        self.name.as_mut()
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

    super::impl_token_fns!(iter = [name, token]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionReturnType {
    Type(Box<Type>),
    TypePack(TypePack),
    GenericTypePack(GenericTypePack),
    VariadicTypePack(VariadicTypePack),
}

impl<T: Into<Type>> From<T> for FunctionReturnType {
    fn from(r#type: T) -> Self {
        match r#type.into() {
            Type::Parenthese(parenthese) => {
                Self::TypePack(TypePack::default().with_type(parenthese.into_inner_type()))
            }
            other => Self::Type(Box::new(other)),
        }
    }
}

impl From<TypePack> for FunctionReturnType {
    fn from(type_pack: TypePack) -> Self {
        Self::TypePack(type_pack)
    }
}

impl From<GenericTypePack> for FunctionReturnType {
    fn from(generic_type_pack: GenericTypePack) -> Self {
        Self::GenericTypePack(generic_type_pack)
    }
}

impl From<VariadicTypePack> for FunctionReturnType {
    fn from(variadic_type_pack: VariadicTypePack) -> Self {
        Self::VariadicTypePack(variadic_type_pack)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariadicArgumentType {
    GenericTypePack(GenericTypePack),
    VariadicTypePack(VariadicTypePack),
}

impl From<GenericTypePack> for VariadicArgumentType {
    fn from(generic_type_pack: GenericTypePack) -> Self {
        Self::GenericTypePack(generic_type_pack)
    }
}

impl From<VariadicTypePack> for VariadicArgumentType {
    fn from(variadic_type_pack: VariadicTypePack) -> Self {
        Self::VariadicTypePack(variadic_type_pack)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionType {
    arguments: Vec<FunctionArgumentType>,
    variadic_argument_type: Option<VariadicArgumentType>,
    return_type: FunctionReturnType,
    generic_parameters: Option<GenericParameters>,
    tokens: Option<FunctionTypeTokens>,
}

impl FunctionType {
    pub fn new(return_type: impl Into<FunctionReturnType>) -> Self {
        Self {
            arguments: Vec::new(),
            variadic_argument_type: None,
            return_type: return_type.into(),
            generic_parameters: None,
            tokens: None,
        }
    }

    pub fn with_generic_parameters(mut self, generic_parameters: GenericParameters) -> Self {
        self.generic_parameters = Some(generic_parameters);
        self
    }

    #[inline]
    pub fn set_generic_parameters(&mut self, generic_parameters: GenericParameters) {
        self.generic_parameters = Some(generic_parameters);
    }

    #[inline]
    pub fn get_generic_parameters(&self) -> Option<&GenericParameters> {
        self.generic_parameters.as_ref()
    }

    pub fn with_argument(mut self, argument: impl Into<FunctionArgumentType>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub fn with_named_argument(
        mut self,
        name: impl Into<Identifier>,
        argument: impl Into<Type>,
    ) -> Self {
        self.arguments
            .push(FunctionArgumentType::new(argument.into()).with_name(name.into()));
        self
    }

    pub fn push_argument(&mut self, argument: impl Into<FunctionArgumentType>) {
        self.arguments.push(argument.into());
    }

    pub fn with_variadic_type(mut self, variadic_type: impl Into<VariadicArgumentType>) -> Self {
        self.variadic_argument_type = Some(variadic_type.into());
        self
    }

    pub fn set_variadic_type(&mut self, variadic_type: impl Into<VariadicArgumentType>) {
        self.variadic_argument_type = Some(variadic_type.into());
    }

    #[inline]
    pub fn get_variadic_argument_type(&self) -> Option<&VariadicArgumentType> {
        self.variadic_argument_type.as_ref()
    }

    #[inline]
    pub fn has_variadic_argument_type(&self) -> bool {
        self.variadic_argument_type.is_some()
    }

    #[inline]
    pub fn mutate_variadic_argument_type(&mut self) -> Option<&mut VariadicArgumentType> {
        self.variadic_argument_type.as_mut()
    }

    #[inline]
    pub fn iter_arguments(&self) -> impl Iterator<Item = &FunctionArgumentType> {
        self.arguments.iter()
    }

    #[inline]
    pub fn iter_mut_arguments(&mut self) -> impl Iterator<Item = &mut FunctionArgumentType> {
        self.arguments.iter_mut()
    }

    #[inline]
    pub fn argument_len(&self) -> usize {
        self.arguments.len()
    }

    #[inline]
    pub fn get_return_type(&self) -> &FunctionReturnType {
        &self.return_type
    }

    #[inline]
    pub fn mutate_return_type(&mut self) -> &mut FunctionReturnType {
        &mut self.return_type
    }

    pub fn with_tokens(mut self, tokens: FunctionTypeTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: FunctionTypeTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&FunctionTypeTokens> {
        self.tokens.as_ref()
    }

    super::impl_token_fns!(iter = [tokens, generic_parameters, arguments]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionTypeTokens {
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
    pub arrow: Token,
    pub commas: Vec<Token>,
}

impl FunctionTypeTokens {
    super::impl_token_fns!(
        target = [opening_parenthese, closing_parenthese, arrow]
        iter = [commas]
    );
}
