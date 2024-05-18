mod array;
mod expression_type;
mod function;
mod function_variadic_type;
mod generics;
mod intersection;
mod optional;
mod parenthese;
mod string_type;
mod table;
mod type_field;
mod type_name;
mod type_pack;
mod union;
mod variadic_type_pack;

pub use array::*;
pub use expression_type::*;
pub use function::*;
pub use function_variadic_type::*;
pub use generics::*;
pub use intersection::*;
pub use optional::*;
pub use parenthese::*;
pub use string_type::*;
pub use table::*;
pub use type_field::*;
pub use type_name::*;
pub use type_pack::*;
pub use union::*;
pub use variadic_type_pack::*;

use crate::nodes::Token;

use super::impl_token_fns;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Name(TypeName),
    Field(TypeField),
    True(Option<Token>),
    False(Option<Token>),
    Nil(Option<Token>),
    String(StringType),
    Array(ArrayType),
    Table(TableType),
    TypeOf(ExpressionType),
    Parenthese(ParentheseType),
    Function(FunctionType),
    Optional(OptionalType),
    Intersection(IntersectionType),
    Union(UnionType),
}

impl Type {
    pub fn nil() -> Self {
        Self::Nil(None)
    }

    pub fn in_parentheses(self) -> Type {
        Self::Parenthese(ParentheseType::new(self))
    }
}

impl From<bool> for Type {
    fn from(value: bool) -> Self {
        match value {
            true => Self::True(None),
            false => Self::False(None),
        }
    }
}

impl<T: Into<Type>> From<Option<T>> for Type {
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::nil(),
            Some(value) => value.into(),
        }
    }
}

impl From<TypeName> for Type {
    fn from(name: TypeName) -> Self {
        Self::Name(name)
    }
}

impl From<TypeField> for Type {
    fn from(type_field: TypeField) -> Self {
        Self::Field(type_field)
    }
}

impl From<FunctionType> for Type {
    fn from(function: FunctionType) -> Self {
        Self::Function(function)
    }
}

impl From<ArrayType> for Type {
    fn from(array: ArrayType) -> Self {
        Self::Array(array)
    }
}

impl From<TableType> for Type {
    fn from(table: TableType) -> Self {
        Self::Table(table)
    }
}

impl From<ExpressionType> for Type {
    fn from(type_of: ExpressionType) -> Self {
        Self::TypeOf(type_of)
    }
}

impl From<ParentheseType> for Type {
    fn from(parenthese_type: ParentheseType) -> Self {
        Self::Parenthese(parenthese_type)
    }
}

impl From<StringType> for Type {
    fn from(string_type: StringType) -> Self {
        Self::String(string_type)
    }
}

impl From<OptionalType> for Type {
    fn from(optional_type: OptionalType) -> Self {
        Self::Optional(optional_type)
    }
}

impl From<IntersectionType> for Type {
    fn from(intersection: IntersectionType) -> Self {
        Self::Intersection(intersection)
    }
}

impl From<UnionType> for Type {
    fn from(string_type: UnionType) -> Self {
        Self::Union(string_type)
    }
}
