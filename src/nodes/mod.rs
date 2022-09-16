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
    AnyBlock(&'a Block),
    AnyExpression(AnyExpressionRef<'a>),
    AnyStatement(AnyStatementRef<'a>),
}

impl<'a> From<&'a Block> for AnyNodeRef<'a> {
    fn from(block: &'a Block) -> Self {
        Self::AnyBlock(block)
    }
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
        Self::AnyExpression(AnyExpressionRef::from(expression))
    }
}

impl<'a> From<&'a Prefix> for AnyNodeRef<'a> {
    fn from(prefix: &'a Prefix) -> Self {
        Self::AnyExpression(AnyExpressionRef::from(prefix))
    }
}

impl<'a> From<&'a Arguments> for AnyNodeRef<'a> {
    fn from(arguments: &'a Arguments) -> Self {
        Self::AnyExpression(AnyExpressionRef::from(arguments))
    }
}

impl<'a> From<&'a Variable> for AnyNodeRef<'a> {
    fn from(variable: &'a Variable) -> Self {
        Self::AnyExpression(AnyExpressionRef::from(variable))
    }
}

impl<'a> From<&'a TableEntry> for AnyNodeRef<'a> {
    fn from(entry: &'a TableEntry) -> Self {
        Self::AnyExpression(AnyExpressionRef::from(entry))
    }
}

impl<'a> From<AnyStatementRef<'a>> for AnyNodeRef<'a> {
    fn from(any_statement: AnyStatementRef<'a>) -> Self {
        Self::AnyStatement(any_statement)
    }
}

impl<'a> From<AnyExpressionRef<'a>> for AnyNodeRef<'a> {
    fn from(any_expression: AnyExpressionRef<'a>) -> Self {
        Self::AnyExpression(any_expression)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnyStatementRef<'a> {
    Statement(&'a Statement),
    LastStatement(&'a LastStatement),
}

impl<'a> From<&'a Statement> for AnyStatementRef<'a> {
    fn from(statement: &'a Statement) -> Self {
        Self::Statement(statement)
    }
}

impl<'a> From<&'a LastStatement> for AnyStatementRef<'a> {
    fn from(statement: &'a LastStatement) -> Self {
        Self::LastStatement(statement)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnyExpressionRef<'a> {
    Expression(&'a Expression),
    Prefix(&'a Prefix),
    Arguments(&'a Arguments),
    Variable(&'a Variable),
    TableEntry(&'a TableEntry),
}

impl<'a> From<&'a Expression> for AnyExpressionRef<'a> {
    fn from(expression: &'a Expression) -> Self {
        Self::Expression(expression)
    }
}

impl<'a> From<&'a Prefix> for AnyExpressionRef<'a> {
    fn from(prefix: &'a Prefix) -> Self {
        Self::Prefix(prefix)
    }
}

impl<'a> From<&'a Arguments> for AnyExpressionRef<'a> {
    fn from(arguments: &'a Arguments) -> Self {
        Self::Arguments(arguments)
    }
}

impl<'a> From<&'a Variable> for AnyExpressionRef<'a> {
    fn from(variable: &'a Variable) -> Self {
        Self::Variable(variable)
    }
}

impl<'a> From<&'a TableEntry> for AnyExpressionRef<'a> {
    fn from(entry: &'a TableEntry) -> Self {
        Self::TableEntry(entry)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AnyNodeRefMut<'a> {
    AnyBlock(&'a mut Block),
    AnyExpression(AnyExpressionRefMut<'a>),
    AnyStatement(AnyStatementRefMut<'a>),
}

impl<'a> From<&'a mut Block> for AnyNodeRefMut<'a> {
    fn from(block: &'a mut Block) -> Self {
        Self::AnyBlock(block)
    }
}

impl<'a> From<&'a mut Statement> for AnyNodeRefMut<'a> {
    fn from(statement: &'a mut Statement) -> Self {
        Self::AnyStatement(AnyStatementRefMut::from(statement))
    }
}

impl<'a> From<&'a mut LastStatement> for AnyNodeRefMut<'a> {
    fn from(statement: &'a mut LastStatement) -> Self {
        Self::AnyStatement(AnyStatementRefMut::from(statement))
    }
}

impl<'a> From<&'a mut Expression> for AnyNodeRefMut<'a> {
    fn from(expression: &'a mut Expression) -> Self {
        Self::AnyExpression(AnyExpressionRefMut::from(expression))
    }
}

impl<'a> From<AnyStatementRefMut<'a>> for AnyNodeRefMut<'a> {
    fn from(any_statement: AnyStatementRefMut<'a>) -> Self {
        Self::AnyStatement(any_statement)
    }
}

impl<'a> From<AnyExpressionRefMut<'a>> for AnyNodeRefMut<'a> {
    fn from(any_expression: AnyExpressionRefMut<'a>) -> Self {
        Self::AnyExpression(any_expression)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AnyStatementRefMut<'a> {
    Statement(&'a mut Statement),
    LastStatement(&'a mut LastStatement),
}

impl<'a> From<&'a mut Statement> for AnyStatementRefMut<'a> {
    fn from(statement: &'a mut Statement) -> Self {
        Self::Statement(statement)
    }
}

impl<'a> From<&'a mut LastStatement> for AnyStatementRefMut<'a> {
    fn from(statement: &'a mut LastStatement) -> Self {
        Self::LastStatement(statement)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AnyExpressionRefMut<'a> {
    Expression(&'a mut Expression),
    Prefix(&'a mut Prefix),
    Arguments(&'a mut Arguments),
    Variable(&'a mut Variable),
    TableEntry(&'a mut TableEntry),
}

impl<'a> From<&'a mut Expression> for AnyExpressionRefMut<'a> {
    fn from(expression: &'a mut Expression) -> Self {
        Self::Expression(expression)
    }
}

impl<'a> From<&'a mut Prefix> for AnyExpressionRefMut<'a> {
    fn from(prefix: &'a mut Prefix) -> Self {
        Self::Prefix(prefix)
    }
}

impl<'a> From<&'a mut Arguments> for AnyExpressionRefMut<'a> {
    fn from(arguments: &'a mut Arguments) -> Self {
        Self::Arguments(arguments)
    }
}

impl<'a> From<&'a mut Variable> for AnyExpressionRefMut<'a> {
    fn from(variable: &'a mut Variable) -> Self {
        Self::Variable(variable)
    }
}

impl<'a> From<&'a mut TableEntry> for AnyExpressionRefMut<'a> {
    fn from(entry: &'a mut TableEntry) -> Self {
        Self::TableEntry(entry)
    }
}
