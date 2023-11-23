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

    pub fn clear_comments(&mut self) {
        if let Some(name) = &mut self.name {
            name.clear_comments();
        }
        if let Some(token) = &mut self.token {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(name) = &mut self.name {
            name.clear_whitespaces();
        }
        if let Some(token) = &mut self.token {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(name) = &mut self.name {
            name.replace_referenced_tokens(code);
        }
        if let Some(token) = &mut self.token {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(name) = &mut self.name {
            name.shift_token_line(amount);
        }
        if let Some(token) = &mut self.token {
            token.shift_token_line(amount);
        }
    }
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
        Self::Type(Box::new(r#type.into()))
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

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
        if let Some(generics) = &mut self.generic_parameters {
            generics.clear_comments();
        }
        for argument in &mut self.arguments {
            argument.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
        if let Some(generics) = &mut self.generic_parameters {
            generics.clear_whitespaces();
        }
        for argument in &mut self.arguments {
            argument.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
        if let Some(generics) = &mut self.generic_parameters {
            generics.replace_referenced_tokens(code);
        }
        for argument in &mut self.arguments {
            argument.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
        if let Some(generics) = &mut self.generic_parameters {
            generics.shift_token_line(amount);
        }
        for argument in &mut self.arguments {
            argument.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionTypeTokens {
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
    pub arrow: Token,
    pub commas: Vec<Token>,
}

impl FunctionTypeTokens {
    pub fn clear_comments(&mut self) {
        self.opening_parenthese.clear_comments();
        self.closing_parenthese.clear_comments();
        self.arrow.clear_comments();
        for comma in self.commas.iter_mut() {
            comma.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_parenthese.clear_whitespaces();
        self.closing_parenthese.clear_whitespaces();
        self.arrow.clear_whitespaces();
        for comma in self.commas.iter_mut() {
            comma.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_parenthese.replace_referenced_tokens(code);
        self.closing_parenthese.replace_referenced_tokens(code);
        self.arrow.replace_referenced_tokens(code);
        for comma in self.commas.iter_mut() {
            comma.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.opening_parenthese.shift_token_line(amount);
        self.closing_parenthese.shift_token_line(amount);
        self.arrow.shift_token_line(amount);
        for comma in self.commas.iter_mut() {
            comma.shift_token_line(amount);
        }
    }
}
