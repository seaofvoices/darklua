use crate::nodes::*;
use crate::process::{NodeProcessor, NodeProcessorMut, NodeVisitor, NodeVisitorMut};

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
    fn insert(&mut self, identifier: &String);
    /// Called when a new local variable is initialized.
    fn insert_local(&mut self, identifier: &String, value: Option<&Expression>);
    /// Called when a new local function is initialized.
    fn insert_local_function(&mut self, function: &LocalFunctionStatement);
}

/// Defines methods to interact with the concept of lexical scoping. The struct implementing this
/// trait should be able to keep track of identifiers when used along the ScopeVisitorMut.
pub trait ScopeMut {
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
    fn visit_block_without_push<T: NodeProcessor + Scope>(block: &Block, scope: &mut T) {
        scope.process_block(block);

        block.get_statements()
            .iter()
            .for_each(|statement| Self::visit_statement(statement, scope));

        if let Some(last_statement) = block.get_last_statement() {
            scope.process_last_statement(last_statement);

            match last_statement {
                LastStatement::Return(expressions) => {
                    expressions.iter()
                        .for_each(|expression| Self::visit_expression(expression, scope));
                }
                _ => {}
            };
        };
    }
}

impl<T: NodeProcessor + Scope> NodeVisitor<T> for ScopeVisitor {
    fn visit_block(block: &Block, scope: &mut T) {
        scope.push();
        Self::visit_block_without_push(block, scope);
        scope.pop();
    }

    fn visit_local_assign(statement: &LocalAssignStatement, scope: &mut T) {
        scope.process_local_assign_statement(statement);

        statement.get_values().iter()
            .for_each(|value| Self::visit_expression(value, scope));

        statement.for_each_assignment(|variable, expression| scope.insert_local(variable, expression));
    }

    fn visit_function_expression(function: &FunctionExpression, scope: &mut T) {
        scope.process_function_expression(function);

        scope.push();
        function.get_parameters().iter()
            .for_each(|parameter| scope.insert(parameter));

        Self::visit_block(function.get_block(), scope);
        scope.pop();
    }

    fn visit_function_statement(statement: &FunctionStatement, scope: &mut T) {
        scope.process_function_statement(statement);
        scope.process_variable_assignment(statement.get_name().get_identifier());

        scope.push();
        statement.get_parameters().iter()
            .for_each(|parameter| scope.insert(parameter));

        Self::visit_block(statement.get_block(), scope);
        scope.pop();
    }

    fn visit_local_function(statement: &LocalFunctionStatement, scope: &mut T) {
        scope.process_local_function_statement(statement);

        scope.insert_local_function(statement);

        scope.push();
        statement.get_parameters().iter()
            .for_each(|parameter| scope.insert(parameter));

        Self::visit_block(statement.get_block(), scope);
        scope.pop();
    }

    fn visit_generic_for(statement: &GenericForStatement, scope: &mut T) {
        scope.process_generic_for_statement(statement);

        statement.get_expressions().iter()
            .for_each(|expression| Self::visit_expression(expression, scope));

        statement.get_identifiers().iter()
            .for_each(|identifier| scope.insert(identifier));

        Self::visit_block(statement.get_block(), scope);
    }

    fn visit_numeric_for(statement: &NumericForStatement, scope: &mut T) {
        scope.process_numeric_for_statement(statement);

        Self::visit_expression(statement.get_start(), scope);
        Self::visit_expression(statement.get_end(), scope);

        if let Some(step) = statement.get_step() {
            Self::visit_expression(step, scope);
        };

        scope.push();
        scope.insert(statement.get_identifier());

        Self::visit_block(statement.get_block(), scope);
        scope.pop();
    }

    fn visit_repeat_statement(statement: &RepeatStatement, scope: &mut T) {
        scope.process_repeat_statement(statement);

        scope.push();

        Self::visit_block_without_push(statement.get_block(), scope);
        Self::visit_expression(statement.get_condition(), scope);

        scope.pop();
    }
}

/// A visitor that can be used only with a NodeProcessorMut that also implements the ScopeMut trait.
pub struct ScopeVisitorMut;

impl ScopeVisitorMut {
    fn visit_block_without_push<T: NodeProcessorMut + ScopeMut>(block: &mut Block, scope: &mut T) {
        scope.process_block(block);

        block.mutate_statements()
            .iter_mut()
            .for_each(|statement| Self::visit_statement(statement, scope));

        if let Some(last_statement) = block.mutate_last_statement() {
            scope.process_last_statement(last_statement);

            match last_statement {
                LastStatement::Return(expressions) => {
                    expressions.iter_mut()
                        .for_each(|expression| Self::visit_expression(expression, scope));
                }
                _ => {}
            };
        };
    }
}

impl<T: NodeProcessorMut + ScopeMut> NodeVisitorMut<T> for ScopeVisitorMut {
    fn visit_block(block: &mut Block, scope: &mut T) {
        scope.push();
        Self::visit_block_without_push(block, scope);
        scope.pop();
    }

    fn visit_local_assign(statement: &mut LocalAssignStatement, scope: &mut T) {
        scope.process_local_assign_statement(statement);

        statement.mutate_values().iter_mut()
            .for_each(|value| Self::visit_expression(value, scope));

        statement.for_each_assignment_mut(|variable, expression| scope.insert_local(variable, expression));
    }

    fn visit_function_expression(function: &mut FunctionExpression, scope: &mut T) {
        scope.process_function_expression(function);

        scope.push();
        function.mutate_parameters().iter_mut()
            .for_each(|parameter| scope.insert(parameter));

        Self::visit_block(function.mutate_block(), scope);
        scope.pop();
    }

    fn visit_function_statement(statement: &mut FunctionStatement, scope: &mut T) {
        scope.process_function_statement(statement);
        scope.process_variable_expression(statement.mutate_function_name().mutate_identifier());

        scope.push();
        statement.mutate_parameters().iter_mut()
            .for_each(|parameter| scope.insert(parameter));

        Self::visit_block(statement.mutate_block(), scope);
        scope.pop();
    }

    fn visit_local_function(statement: &mut LocalFunctionStatement, scope: &mut T) {
        scope.process_local_function_statement(statement);

        scope.insert_local_function(statement);

        scope.push();
        statement.mutate_parameters().iter_mut()
            .for_each(|parameter| scope.insert(parameter));

        Self::visit_block(statement.mutate_block(), scope);
        scope.pop();
    }

    fn visit_generic_for(statement: &mut GenericForStatement, scope: &mut T) {
        scope.process_generic_for_statement(statement);

        statement.mutate_expressions().iter_mut()
            .for_each(|expression| Self::visit_expression(expression, scope));

        statement.mutate_identifiers().iter_mut()
            .for_each(|identifier| scope.insert(identifier));

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
        scope.insert(statement.mutate_identifier());

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
