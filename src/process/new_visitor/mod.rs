mod node_stack;
mod statement_visitor;

use node_stack::NodeStack;
pub use statement_visitor::StatementVisitor;

use crate::nodes::{
    AnyExpressionRef, AnyNodeRef, AnyStatementRef, Block, LastStatement, Statement,
};

use super::{mutation::MutationResolver, path::NodePath};

pub fn visit_statements<Arguments, Ret, V>(block: &Block, visitor: V) -> MutationResolver
where
    V: StatementVisitor<Arguments, Ret, ()>,
{
    let mut context = ();
    visit_statements_context(block, visitor, &mut context)
}

pub fn visit_statements_context<Arguments, Context, Ret, V>(
    block: &Block,
    visitor: V,
    context: &mut Context,
) -> MutationResolver
where
    V: StatementVisitor<Arguments, Ret, Context>,
{
    let mut mutations = MutationResolver::default();
    let mut node_stack = NodeStack::new(block);

    while let Some((node, path)) = node_stack.pop() {
        match node {
            AnyNodeRef::AnyBlock(block) => {
                for (index, statement) in block.iter_statements().enumerate() {
                    node_stack.push(statement, path.join_statement(index));
                }
                if let Some(last_statement) = block.get_last_statement() {
                    node_stack.push(
                        last_statement,
                        path.join_statement(block.total_len().saturating_sub(1)),
                    );
                }
            }
            AnyNodeRef::AnyStatement(any_statement) => match any_statement {
                AnyStatementRef::Statement(statement) => {
                    if let Some(mutation) = visitor.statement(statement, &path, context) {
                        mutations.add(mutation);
                    }

                    match statement {
                        Statement::Assign(assign) => {
                            for (index, variable) in assign.iter_variables().enumerate() {
                                node_stack.push(variable, path.join_expression(index));
                            }
                            node_stack.push_expressions(assign.iter_values(), &path);
                        }
                        Statement::Call(call) => {
                            node_stack.push(call.get_arguments(), path);
                        }
                        Statement::CompoundAssign(assign) => {
                            node_stack.push(assign.get_variable(), path.join_expression(0));
                            node_stack.push(assign.get_value(), path.join_expression(1));
                        }
                        Statement::Do(do_statement) => {
                            node_stack.push(do_statement.get_block(), path.join_block(0));
                        }
                        Statement::Function(function) => {
                            node_stack.push(function.get_block(), path.join_block(0));
                        }
                        Statement::GenericFor(generic_for) => {
                            node_stack.push_expressions(generic_for.iter_expressions(), &path);
                            node_stack.push(generic_for.get_block(), path.join_block(0));
                        }
                        Statement::If(if_statement) => {
                            for (index, branch) in if_statement.iter_branches().enumerate() {
                                node_stack
                                    .push(branch.get_condition(), path.join_expression(index));
                                node_stack.push(branch.get_block(), path.join_block(index));
                            }
                        }
                        Statement::LocalAssign(assign) => {
                            node_stack.push_expressions(assign.iter_values(), &path);
                        }
                        Statement::LocalFunction(function) => {
                            node_stack.push(function.get_block(), path.join_block(0));
                        }
                        Statement::NumericFor(numeric_for) => {
                            node_stack.push(numeric_for.get_start(), path.join_expression(0));
                            node_stack.push(numeric_for.get_end(), path.join_expression(1));
                            if let Some(step) = numeric_for.get_step() {
                                node_stack.push(step, path.join_expression(2));
                            }
                            node_stack.push(numeric_for.get_block(), path.join_block(0));
                        }
                        Statement::Repeat(repeat) => {
                            node_stack.push(repeat.get_block(), path.join_block(0));
                            node_stack.push(repeat.get_condition(), path.join_expression(0));
                        }
                        Statement::While(while_statement) => {
                            node_stack
                                .push(while_statement.get_condition(), path.join_expression(0));
                            node_stack.push(while_statement.get_block(), path.join_block(0));
                        }
                    }
                }
                AnyStatementRef::LastStatement(last_statement) => {
                    if let Some(mutation) = visitor.last_statement(last_statement, &path, context) {
                        mutations.add(mutation);
                    }

                    match last_statement {
                        LastStatement::Break(_) | LastStatement::Continue(_) => {}
                        LastStatement::Return(return_statement) => {
                            node_stack.push_expressions(return_statement.iter_expressions(), &path);
                        }
                    }
                }
            },
            AnyNodeRef::AnyExpression(any_expression) => match any_expression {
                AnyExpressionRef::Expression(_) => todo!(),
                AnyExpressionRef::Prefix(_) => todo!(),
                AnyExpressionRef::Arguments(_) => todo!(),
                AnyExpressionRef::Variable(_) => todo!(),
            },
        }
    }

    mutations
}

#[cfg(test)]
mod test {
    use crate::{
        process::{
            mutation::{Mutation, StatementRemoval},
            path::NodePathSlice,
        },
        Parser,
    };

    use super::*;

    #[test]
    fn test_something() {
        let visitor = |statement: &Statement, path: &NodePathSlice| -> Option<Mutation> {
            if let Statement::Do(do_statement) = statement {
                if do_statement.get_block().is_empty() {
                    return Some(StatementRemoval::remove(path.to_path_buf()).into());
                }
            }
            None
        };

        let parser = Parser::default();
        let mut block = parser
            .parse("do end")
            .expect("given test code should parse");

        let mutations = visit_statements(&block, visitor);

        mutations
            .resolve(&mut block)
            .expect("mutations should resolve without errors");

        let expect_block = parser.parse("").expect("given test code should parse");

        pretty_assertions::assert_eq!(block, expect_block);
    }

    #[test]
    fn test_something_with_context() {
        struct Counter {
            counter: usize,
        }
        let visitor = |statement: &Statement, c: &mut Counter| {
            if let Statement::Do(do_statement) = statement {
                if do_statement.get_block().is_empty() {
                    c.counter += 1;
                }
            }
        };

        let parser = Parser::default();
        let block = parser
            .parse("do end do end do do end end")
            .expect("given test code should parse");

        let mut counter = Counter { counter: 0 };

        let mutations = visit_statements_context(&block, visitor, &mut counter);

        assert_eq!(counter.counter, 3);
        assert!(
            mutations.is_empty(),
            "there should not be any mutations in {:?}",
            mutations
        );
    }
}
