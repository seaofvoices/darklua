use crate::nodes::{AnyNodeRef, AnyStatementRef, Block, Expression, Statement};

#[derive(Clone, Debug)]
enum Component {
    Statement(usize),
    Expression(usize),
}

#[derive(Clone, Debug)]
pub struct StatementPath {
    components: Vec<Component>,
}

impl StatementPath {
    fn resolve<'a>(&self, block: &'a Block) -> Option<AnyStatementRef<'a>> {
        let length = self.components.len();

        let mut components = self.components.get(0..length.checked_sub(1)?)?.iter();

        let mut current = match components.next()? {
            Component::Statement(index) => AnyNodeRef::from(block.get_statement(*index)?),
            Component::Expression(_) => return None,
        };

        loop {
            current = match components.next()? {
                Component::Statement(index) => {
                    let next_statement = match current {
                        AnyNodeRef::AnyStatement(statement) => match statement {
                            AnyStatementRef::Statement(statement) => match statement {
                                Statement::Do(do_statement) => {
                                    do_statement.get_block().get_statement(*index)?
                                }
                                Statement::Function(function) => {
                                    function.get_block().get_statement(*index)?
                                }
                                Statement::GenericFor(generic_for) => {
                                    generic_for.get_block().get_statement(*index)?
                                }
                                Statement::If(if_statement) => {
                                    if_statement.get_statement(*index)?
                                }
                                Statement::LocalFunction(function) => {
                                    function.get_block().get_statement(*index)?
                                }
                                Statement::NumericFor(numeric_for) => {
                                    numeric_for.get_block().get_statement(*index)?
                                }
                                Statement::Repeat(repeat) => {
                                    repeat.get_block().get_statement(*index)?
                                }
                                Statement::While(while_statement) => {
                                    while_statement.get_block().get_statement(*index)?
                                }
                                Statement::Assign(_)
                                | Statement::Call(_)
                                | Statement::CompoundAssign(_)
                                | Statement::LocalAssign(_) => return None,
                            },

                            AnyStatementRef::LastStatement(_) => return None,
                        },
                        AnyNodeRef::Expression(expression) => match expression {
                            Expression::Function(function) => {
                                function.get_block().get_statement(*index)?
                            }
                            Expression::Binary(_)
                            | Expression::Call(_)
                            | Expression::False(_)
                            | Expression::Field(_)
                            | Expression::Identifier(_)
                            | Expression::If(_)
                            | Expression::Index(_)
                            | Expression::Nil(_)
                            | Expression::Number(_)
                            | Expression::Parenthese(_)
                            | Expression::String(_)
                            | Expression::Table(_)
                            | Expression::True(_)
                            | Expression::Unary(_)
                            | Expression::VariableArguments(_) => return None,
                        },
                    };
                    AnyNodeRef::from(next_statement)
                }
                Component::Expression(_index) => {}
            }
        }

        Some(current)
    }
}

#[derive(Clone, Debug)]
pub struct StatementSpan {}

#[derive(Clone, Debug)]
pub enum StatementMutation {
    Remove(StatementSpan),
    Replace(StatementSpan, Vec<Statement>),
    Insert(StatementPath, Vec<Statement>),
}
