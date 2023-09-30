mod array;
mod expression_type;
mod function;
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

    pub fn clear_comments(&mut self) {
        match self {
            Type::Name(name) => name.clear_comments(),
            Type::Field(field) => field.clear_comments(),
            Type::True(token) | Type::False(token) | Type::Nil(token) => {
                if let Some(token) = token {
                    token.clear_comments();
                }
            }
            Type::String(string) => string.clear_comments(),
            Type::Array(array) => array.clear_comments(),
            Type::Table(table) => table.clear_comments(),
            Type::TypeOf(expression_type) => expression_type.clear_comments(),
            Type::Parenthese(parenthese) => parenthese.clear_comments(),
            Type::Function(function) => function.clear_comments(),
            Type::Optional(optional) => optional.clear_comments(),
            Type::Intersection(intersection) => intersection.clear_comments(),
            Type::Union(union) => union.clear_comments(),
        }
    }

    pub fn clear_whitespaces(&mut self) {
        match self {
            Type::Name(name) => name.clear_whitespaces(),
            Type::Field(field) => field.clear_whitespaces(),
            Type::True(token) | Type::False(token) | Type::Nil(token) => {
                if let Some(token) = token {
                    token.clear_whitespaces();
                }
            }
            Type::String(string) => string.clear_whitespaces(),
            Type::Array(array) => array.clear_whitespaces(),
            Type::Table(table) => table.clear_whitespaces(),
            Type::TypeOf(expression_type) => expression_type.clear_whitespaces(),
            Type::Parenthese(parenthese) => parenthese.clear_whitespaces(),
            Type::Function(function) => function.clear_whitespaces(),
            Type::Optional(optional) => optional.clear_whitespaces(),
            Type::Intersection(intersection) => intersection.clear_whitespaces(),
            Type::Union(union) => union.clear_whitespaces(),
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        match self {
            Type::Name(name) => name.replace_referenced_tokens(code),
            Type::Field(field) => field.replace_referenced_tokens(code),
            Type::True(token) | Type::False(token) | Type::Nil(token) => {
                if let Some(token) = token {
                    token.replace_referenced_tokens(code);
                }
            }
            Type::String(string) => string.replace_referenced_tokens(code),
            Type::Array(array) => array.replace_referenced_tokens(code),
            Type::Table(table) => table.replace_referenced_tokens(code),
            Type::TypeOf(expression_type) => expression_type.replace_referenced_tokens(code),
            Type::Parenthese(parenthese) => parenthese.replace_referenced_tokens(code),
            Type::Function(function) => function.replace_referenced_tokens(code),
            Type::Optional(optional) => optional.replace_referenced_tokens(code),
            Type::Intersection(intersection) => intersection.replace_referenced_tokens(code),
            Type::Union(union) => union.replace_referenced_tokens(code),
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        match self {
            Type::Name(name) => name.shift_token_line(amount),
            Type::Field(field) => field.shift_token_line(amount),
            Type::True(token) | Type::False(token) | Type::Nil(token) => {
                if let Some(token) = token {
                    token.shift_token_line(amount);
                }
            }
            Type::String(string) => string.shift_token_line(amount),
            Type::Array(array) => array.shift_token_line(amount),
            Type::Table(table) => table.shift_token_line(amount),
            Type::TypeOf(expression_type) => expression_type.shift_token_line(amount),
            Type::Parenthese(parenthese) => parenthese.shift_token_line(amount),
            Type::Function(function) => function.shift_token_line(amount),
            Type::Optional(optional) => optional.shift_token_line(amount),
            Type::Intersection(intersection) => intersection.shift_token_line(amount),
            Type::Union(union) => union.shift_token_line(amount),
        }
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
