//! The collection of nodes used for the Lua abstract syntax tree.

mod arguments;
mod block;
mod expressions;
mod function_call;
mod identifier;
mod statements;
mod token;
mod variable;

pub use arguments::*;
pub use block::*;
pub use expressions::*;
pub use function_call::*;
pub use identifier::*;
pub use statements::*;
pub use token::*;
pub use variable::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnyNodeRef<'a> {
    AnyStatement(AnyStatementRef<'a>),
    Expression(&'a Expression),
}

impl<'a> AnyNodeRef<'a> {
    // pub fn statement(self) -> Option<AnyStatementRef<'a>> {
    //     match self {
    //         Self::AnyStatement(any_statement) => Some(any_statement),
    //         Self::Expression(_) => None,
    //     }
    // }
}

impl<'a> From<&'a Statement> for AnyNodeRef<'a> {
    fn from(statement: &'a Statement) -> Self {
        Self::AnyStatement(AnyStatementRef::from(statement))
    }
}

impl<'a> From<&'a LastStatement> for AnyNodeRef<'a> {
    fn from(statement: &'a LastStatement) -> Self {
        Self::AnyStatement(AnyStatementRef::from(statement))
    }
}

impl<'a> From<&'a Expression> for AnyNodeRef<'a> {
    fn from(expression: &'a Expression) -> Self {
        Self::Expression(expression)
    }
}

impl<'a> From<AnyStatementRef<'a>> for AnyNodeRef<'a> {
    fn from(any_statement: AnyStatementRef<'a>) -> Self {
        Self::AnyStatement(any_statement)
    }
}
