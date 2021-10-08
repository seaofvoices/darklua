use crate::nodes::*;
use crate::process::{NodeProcessor, NodeVisitor};

/// Defines methods to interact with the concept of lexical scoping. The struct implementing this
/// trait should be able to keep track of identifiers when used along the ScopeVisitor.
pub trait Scope {
    /// This method is called when a new block is entered.
    fn push(&mut self);
    /// When a block is left, this method should should free all identifiers inserted in the
    /// previous block.
    fn pop(&mut self);
    /// Called when entering a function block (with each parameters of the function), with the
    /// identifiers from a generic for statement or the identifier from a numeric for loop.
    fn insert(&mut self, identifier: &mut String);
    /// Called when a new local variable is initialized.
    fn insert_local(&mut self, identifier: &mut String, value: Option<&mut Expression>);
    /// Called when a new local function is initialized.
    fn insert_local_function(&mut self, function: &mut LocalFunctionStatement);
}

/// A visitor that can be used only with a NodeProcessor that also implements the Scope trait.
pub struct ScopeVisitor;

impl ScopeVisitor {
    fn visit_block_without_push<T: NodeProcessor + Scope>(block: &mut Block, scope: &mut T) {
        scope.process_block(block);

        block
            .mutate_statements()
            .iter_mut()
            .for_each(|statement| Self::visit_statement(statement, scope));

        if let Some(last_statement) = block.mutate_last_statement() {
            scope.process_last_statement(last_statement);

            if let LastStatement::Return(expressions) = last_statement {
                expressions
                    .iter_mut_expressions()
                    .for_each(|expression| Self::visit_expression(expression, scope));
            };
        };
    }
}

impl<T: NodeProcessor + Scope> NodeVisitor<T> for ScopeVisitor {
    fn visit_block(block: &mut Block, scope: &mut T) {
        scope.push();
        Self::visit_block_without_push(block, scope);
        scope.pop();
    }

    fn visit_local_assign(statement: &mut LocalAssignStatement, scope: &mut T) {
        scope.process_local_assign_statement(statement);

        statement
            .iter_mut_values()
            .for_each(|value| Self::visit_expression(value, scope));

        statement.for_each_assignment(|variable, expression| {
            scope.insert_local(variable.mutate_name(), expression)
        });
    }

    fn visit_function_expression(function: &mut FunctionExpression, scope: &mut T) {
        scope.process_function_expression(function);

        scope.push();
        function
            .mutate_parameters()
            .iter_mut()
            .for_each(|parameter| scope.insert(parameter.mutate_name()));

        Self::visit_block(function.mutate_block(), scope);
        scope.pop();
    }

    fn visit_function_statement(statement: &mut FunctionStatement, scope: &mut T) {
        scope.process_function_statement(statement);
        scope.process_variable_expression(statement.mutate_function_name().mutate_identifier());

        scope.push();
        statement
            .mutate_parameters()
            .iter_mut()
            .for_each(|parameter| scope.insert(parameter.mutate_name()));

        Self::visit_block(statement.mutate_block(), scope);
        scope.pop();
    }

    fn visit_local_function(statement: &mut LocalFunctionStatement, scope: &mut T) {
        scope.process_local_function_statement(statement);

        scope.insert_local_function(statement);

        scope.push();
        statement
            .mutate_parameters()
            .iter_mut()
            .for_each(|parameter| scope.insert(parameter.mutate_name()));

        Self::visit_block(statement.mutate_block(), scope);
        scope.pop();
    }

    fn visit_generic_for(statement: &mut GenericForStatement, scope: &mut T) {
        scope.process_generic_for_statement(statement);

        statement
            .iter_mut_expressions()
            .for_each(|expression| Self::visit_expression(expression, scope));

        statement
            .iter_mut_identifiers()
            .for_each(|identifier| scope.insert(identifier.mutate_name()));

        Self::visit_block(statement.mutate_block(), scope);
    }

    fn visit_numeric_for(statement: &mut NumericForStatement, scope: &mut T) {
        scope.process_numeric_for_statement(statement);

        Self::visit_expression(statement.mutate_start(), scope);
        Self::visit_expression(statement.mutate_end(), scope);

        if let Some(step) = statement.mutate_step() {
            Self::visit_expression(step, scope);
        };

        scope.push();
        scope.insert(statement.mutate_identifier().mutate_name());

        Self::visit_block(statement.mutate_block(), scope);
        scope.pop();
    }

    fn visit_repeat_statement(statement: &mut RepeatStatement, scope: &mut T) {
        scope.process_repeat_statement(statement);

        scope.push();

        Self::visit_block_without_push(statement.mutate_block(), scope);
        Self::visit_expression(statement.mutate_condition(), scope);

        scope.pop();
    }
}
