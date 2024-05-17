mod assign;
mod compound_assign;
mod do_statement;
mod function;
mod generic_for;
mod if_statement;
mod last_statement;
mod local_assign;
mod local_function;
mod numeric_for;
mod repeat_statement;
mod type_declaration;
mod while_statement;

pub use assign::*;
pub use compound_assign::*;
pub use do_statement::*;
pub use function::*;
pub use generic_for::*;
pub use if_statement::*;
pub use last_statement::*;
pub use local_assign::*;
pub use local_function::*;
pub use numeric_for::*;
pub use repeat_statement::*;
pub use type_declaration::*;
pub use while_statement::*;

use crate::nodes::FunctionCall;

use super::impl_token_fns;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement {
    Assign(AssignStatement),
    Do(DoStatement),
    Call(FunctionCall),
    CompoundAssign(CompoundAssignStatement),
    Function(FunctionStatement),
    GenericFor(GenericForStatement),
    If(IfStatement),
    LocalAssign(LocalAssignStatement),
    LocalFunction(LocalFunctionStatement),
    NumericFor(Box<NumericForStatement>),
    Repeat(RepeatStatement),
    While(WhileStatement),
    TypeDeclaration(TypeDeclarationStatement),
}

impl From<AssignStatement> for Statement {
    fn from(assign: AssignStatement) -> Statement {
        Statement::Assign(assign)
    }
}

impl From<DoStatement> for Statement {
    fn from(do_statement: DoStatement) -> Statement {
        Statement::Do(do_statement)
    }
}

impl From<CompoundAssignStatement> for Statement {
    fn from(statement: CompoundAssignStatement) -> Statement {
        Statement::CompoundAssign(statement)
    }
}

impl From<FunctionCall> for Statement {
    fn from(call: FunctionCall) -> Statement {
        Statement::Call(call)
    }
}

impl From<FunctionStatement> for Statement {
    fn from(function: FunctionStatement) -> Statement {
        Statement::Function(function)
    }
}

impl From<GenericForStatement> for Statement {
    fn from(generic_for: GenericForStatement) -> Statement {
        Statement::GenericFor(generic_for)
    }
}

impl From<IfStatement> for Statement {
    fn from(if_statement: IfStatement) -> Statement {
        Statement::If(if_statement)
    }
}

impl From<LocalAssignStatement> for Statement {
    fn from(assign: LocalAssignStatement) -> Statement {
        Statement::LocalAssign(assign)
    }
}

impl From<LocalFunctionStatement> for Statement {
    fn from(function: LocalFunctionStatement) -> Statement {
        Statement::LocalFunction(function)
    }
}

impl From<NumericForStatement> for Statement {
    fn from(numeric_for: NumericForStatement) -> Statement {
        Statement::NumericFor(numeric_for.into())
    }
}

impl From<RepeatStatement> for Statement {
    fn from(repeat_statement: RepeatStatement) -> Statement {
        Statement::Repeat(repeat_statement)
    }
}

impl From<WhileStatement> for Statement {
    fn from(while_statement: WhileStatement) -> Statement {
        Statement::While(while_statement)
    }
}

impl From<TypeDeclarationStatement> for Statement {
    fn from(type_declaration: TypeDeclarationStatement) -> Statement {
        Statement::TypeDeclaration(type_declaration)
    }
}
