use super::{GenericTypePack, Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionVariadicType {
    Type(Type),
    GenericTypePack(GenericTypePack),
}

impl From<GenericTypePack> for FunctionVariadicType {
    fn from(value: GenericTypePack) -> Self {
        Self::GenericTypePack(value)
    }
}

impl<T: Into<Type>> From<T> for FunctionVariadicType {
    fn from(value: T) -> Self {
        Self::Type(value.into())
    }
}
