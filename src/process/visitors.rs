use crate::nodes::*;
use crate::process::{NodeProcessor, NodeProcessorMut};

use std::marker::PhantomData;

/// A trait that defines method that iterates on nodes and process them using a NodeProcessor.
pub trait NodeVisitor<T: NodeProcessor> {
    fn visit_block(block: &Block, processor: &mut T) {
        processor.process_block(block);

        block.get_statements()
            .iter()
            .for_each(|statement| Self::visit_statement(statement, processor));

        if let Some(last_statement) = block.get_last_statement() {
            Self::visit_last_statement(last_statement, processor);
        }
    }

    fn visit_statement(statement: &Statement, processor: &mut T) {
        processor.process_statement(statement);

        match statement {
            Statement::Assign(statement) => Self::visit_assign_statement(statement, processor),
            Statement::Do(statement) => Self::visit_do_statement(statement, processor),
            Statement::Call(statement) => Self::visit_function_call(statement, processor),
            Statement::Function(statement) => Self::visit_function_statement(statement, processor),
            Statement::GenericFor(statement) => Self::visit_generic_for(statement, processor),
            Statement::If(statement) => Self::visit_if_statement(statement, processor),
            Statement::LocalAssign(statement) => Self::visit_local_assign(statement, processor),
            Statement::LocalFunction(statement) => Self::visit_local_function(statement, processor),
            Statement::NumericFor(statement) => Self::visit_numeric_for(statement, processor),
            Statement::Repeat(statement) => Self::visit_repeat_statement(statement, processor),
            Statement::While(statement) => Self::visit_while_statement(statement, processor),
        };
    }

    fn visit_last_statement(last_statement: &LastStatement, processor: &mut T) {
        processor.process_last_statement(last_statement);

        match last_statement {
            LastStatement::Return(expressions) => {
                expressions.iter()
                    .for_each(|expression| Self::visit_expression(expression, processor));
            }
            LastStatement::Break => {}
        };
    }

    fn visit_expression(expression: &Expression, processor: &mut T) {
        processor.process_expression(expression);

        match expression {
            Expression::Binary(expression) => {
                processor.process_binary_expression(expression);
                Self::visit_expression(expression.left(), processor);
                Self::visit_expression(expression.right(), processor);
            }
            Expression::Call(expression) => Self::visit_function_call(expression, processor),
            Expression::Field(field) => Self::visit_field_expression(field, processor),
            Expression::Function(function) => Self::visit_function_expression(function, processor),
            Expression::Identifier(identifier) => processor.process_variable_expression(identifier),
            Expression::Index(index) => Self::visit_index_expression(index, processor),
            Expression::Number(number) => processor.process_number_expression(number),
            Expression::Parenthese(expression) => Self::visit_expression(expression, processor),
            Expression::String(string) => processor.process_string_expression(string),
            Expression::Table(table) => Self::visit_table(table, processor),
            Expression::Unary(unary) => {
                processor.process_unary_expression(unary);
                Self::visit_expression(unary.get_expression(), processor);
            }
            Expression::False
            | Expression::Nil
            | Expression::True
            | Expression::VariableArguments => {}
        }
    }

    fn visit_function_expression(function: &FunctionExpression, processor: &mut T) {
        processor.process_function_expression(function);

        Self::visit_block(function.get_block(), processor);
    }

    fn visit_assign_statement(statement: &AssignStatement, processor: &mut T) {
        processor.process_assign_statement(statement);

        statement.get_variables().iter()
            .for_each(|variable| match variable {
                Variable::Identifier(identifier) => processor.process_variable_assignment(identifier),
                Variable::Field(field) => Self::visit_field_expression(field, processor),
                Variable::Index(index) => Self::visit_index_expression(index, processor),
            });

        statement.get_values().iter()
            .for_each(|expression| Self::visit_expression(expression, processor));
    }

    fn visit_do_statement(statement: &DoStatement, processor: &mut T) {
        processor.process_do_statement(statement);
        Self::visit_block(statement.get_block(), processor);
    }

    fn visit_function_statement(statement: &FunctionStatement, processor: &mut T) {
        processor.process_function_statement(statement);
        processor.process_variable_assignment(statement.get_name().get_identifier());
        Self::visit_block(statement.get_block(), processor);
    }

    fn visit_generic_for(statement: &GenericForStatement, processor: &mut T) {
        processor.process_generic_for_statement(statement);

        statement.get_expressions().iter()
            .for_each(|expression| Self::visit_expression(expression, processor));
        Self::visit_block(statement.get_block(), processor);
    }

    fn visit_if_statement(statement: &IfStatement, processor: &mut T) {
        processor.process_if_statement(statement);

        statement.get_branches()
            .iter()
            .for_each(|branch| {
                Self::visit_expression(branch.get_condition(), processor);
                Self::visit_block(branch.get_block(), processor);
            });

        if let Some(block) = statement.get_else_block() {
            Self::visit_block(block, processor);
        }
    }

    fn visit_local_assign(statement: &LocalAssignStatement, processor: &mut T) {
        processor.process_local_assign_statement(statement);

        statement.get_values().iter()
            .for_each(|value| Self::visit_expression(value, processor));
    }

    fn visit_local_function(statement: &LocalFunctionStatement, processor: &mut T) {
        processor.process_local_function_statement(statement);
        Self::visit_block(statement.get_block(), processor);
    }

    fn visit_numeric_for(statement: &NumericForStatement, processor: &mut T) {
        processor.process_numeric_for_statement(statement);

        Self::visit_expression(statement.get_start(), processor);
        Self::visit_expression(statement.get_end(), processor);

        if let Some(step) = statement.get_step() {
            Self::visit_expression(step, processor);
        };

        Self::visit_block(statement.get_block(), processor);
    }

    fn visit_repeat_statement(statement: &RepeatStatement, processor: &mut T) {
        processor.process_repeat_statement(statement);

        Self::visit_expression(statement.get_condition(), processor);
        Self::visit_block(statement.get_block(), processor);
    }

    fn visit_while_statement(statement: &WhileStatement, processor: &mut T) {
        processor.process_while_statement(statement);

        Self::visit_expression(statement.get_condition(), processor);
        Self::visit_block(statement.get_block(), processor);
    }

    fn visit_field_expression(field: &FieldExpression, processor: &mut T) {
        processor.process_field_expression(field);

        Self::visit_prefix_expression(field.get_prefix(), processor);
    }

    fn visit_index_expression(index: &IndexExpression, processor: &mut T) {
        processor.process_index_expression(index);

        Self::visit_prefix_expression(index.get_prefix(), processor);
        Self::visit_expression(index.get_index(), processor);
    }

    fn visit_function_call(call: &FunctionCall, processor: &mut T) {
        processor.process_function_call(call);

        Self::visit_prefix_expression(call.get_prefix(), processor);
        Self::visit_arguments(call.get_arguments(), processor);
    }

    fn visit_arguments(arguments: &Arguments, processor: &mut T) {
        match arguments {
            Arguments::String(string) => processor.process_string_expression(string),
            Arguments::Table(table) => Self::visit_table(table, processor),
            Arguments::Tuple(expressions) => expressions.iter()
                .for_each(|expression| Self::visit_expression(expression, processor)),
        }
    }

    fn visit_table(table: &TableExpression, processor: &mut T) {
        processor.process_table_expression(table);

        table.get_entries().iter()
            .for_each(|entry| match entry {
                TableEntry::Field(_field, value) => Self::visit_expression(value, processor),
                TableEntry::Index(key, value) => {
                    Self::visit_expression(key, processor);
                    Self::visit_expression(value, processor);
                }
                TableEntry::Value(value) => Self::visit_expression(value, processor),
            });
    }

    fn visit_prefix_expression(prefix: &Prefix, processor: &mut T) {
        processor.process_prefix_expression(prefix);

        match prefix {
            Prefix::Call(call) => Self::visit_function_call(call, processor),
            Prefix::Field(field) => Self::visit_field_expression(field, processor),
            Prefix::Identifier(identifier) => processor.process_variable_expression(identifier),
            Prefix::Index(index) => Self::visit_index_expression(index, processor),
            Prefix::Parenthese(expression) => Self::visit_expression(expression, processor),
        };
    }
}

/// A trait that defines method that iterates on nodes and process them using a NodeProcessorMut.
pub trait NodeVisitorMut<T: NodeProcessorMut> {
    fn visit_block(block: &mut Block, processor: &mut T) {
        processor.process_block(block);

        block.mutate_statements()
            .iter_mut()
            .for_each(|statement| Self::visit_statement(statement, processor));

        if let Some(last_statement) = block.mutate_last_statement() {
            Self::visit_last_statement(last_statement, processor);
        }
    }

    fn visit_statement(statement: &mut Statement, processor: &mut T) {
        processor.process_statement(statement);

        match statement {
            Statement::Assign(statement) => Self::visit_assign_statement(statement, processor),
            Statement::Do(statement) => Self::visit_do_statement(statement, processor),
            Statement::Call(statement) => Self::visit_function_call(statement, processor),
            Statement::Function(statement) => Self::visit_function_statement(statement, processor),
            Statement::GenericFor(statement) => Self::visit_generic_for(statement, processor),
            Statement::If(statement) => Self::visit_if_statement(statement, processor),
            Statement::LocalAssign(statement) => Self::visit_local_assign(statement, processor),
            Statement::LocalFunction(statement) => Self::visit_local_function(statement, processor),
            Statement::NumericFor(statement) => Self::visit_numeric_for(statement, processor),
            Statement::Repeat(statement) => Self::visit_repeat_statement(statement, processor),
            Statement::While(statement) => Self::visit_while_statement(statement, processor),
        };
    }

    fn visit_last_statement(last_statement: &mut LastStatement, processor: &mut T) {
        processor.process_last_statement(last_statement);

        match last_statement {
            LastStatement::Return(expressions) => {
                expressions.iter_mut()
                    .for_each(|expression| Self::visit_expression(expression, processor));
            }
            LastStatement::Break => {}
        };
    }

    fn visit_expression(expression: &mut Expression, processor: &mut T) {
        processor.process_expression(expression);

        match expression {
            Expression::Binary(expression) => {
                processor.process_binary_expression(expression);
                Self::visit_expression(expression.mutate_left(), processor);
                Self::visit_expression(expression.mutate_right(), processor);
            }
            Expression::Call(expression) => Self::visit_function_call(expression, processor),
            Expression::Field(field) => Self::visit_field_expression(field, processor),
            Expression::Function(function) => Self::visit_function_expression(function, processor),
            Expression::Identifier(identifier) => processor.process_variable_expression(identifier),
            Expression::Index(index) => Self::visit_index_expression(index, processor),
            Expression::Number(number) => processor.process_number_expression(number),
            Expression::Parenthese(expression) => Self::visit_expression(expression, processor),
            Expression::String(string) => processor.process_string_expression(string),
            Expression::Table(table) => Self::visit_table(table, processor),
            Expression::Unary(unary) => {
                processor.process_unary_expression(unary);
                Self::visit_expression(unary.mutate_expression(), processor);
            }
            Expression::False
            | Expression::Nil
            | Expression::True
            | Expression::VariableArguments => {}
        }
    }

    fn visit_function_expression(function: &mut FunctionExpression, processor: &mut T) {
        processor.process_function_expression(function);

        Self::visit_block(function.mutate_block(), processor);
    }

    fn visit_assign_statement(statement: &mut AssignStatement, processor: &mut T) {
        processor.process_assign_statement(statement);

        statement.mutate_variables().iter_mut()
            .for_each(|variable| match variable {
                Variable::Identifier(identifier) => processor.process_variable_assignment(identifier),
                Variable::Field(field) => Self::visit_field_expression(field, processor),
                Variable::Index(index) => Self::visit_index_expression(index, processor),
            });

        statement.mutate_values().iter_mut()
            .for_each(|expression| Self::visit_expression(expression, processor));
    }

    fn visit_do_statement(statement: &mut DoStatement, processor: &mut T) {
        processor.process_do_statement(statement);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_function_statement(statement: &mut FunctionStatement, processor: &mut T) {
        processor.process_function_statement(statement);
        processor.process_variable_assignment(statement.mutate_function_name().mutate_identifier());
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_generic_for(statement: &mut GenericForStatement, processor: &mut T) {
        processor.process_generic_for_statement(statement);

        statement.mutate_expressions().iter_mut()
            .for_each(|expression| Self::visit_expression(expression, processor));
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_if_statement(statement: &mut IfStatement, processor: &mut T) {
        processor.process_if_statement(statement);

        statement.mutate_branches()
            .iter_mut()
            .for_each(|branch| {
                Self::visit_expression(branch.mutate_condition(), processor);
                Self::visit_block(branch.mutate_block(), processor);
            });

        if let Some(block) = statement.mutate_else_block() {
            Self::visit_block(block, processor);
        }
    }

    fn visit_local_assign(statement: &mut LocalAssignStatement, processor: &mut T) {
        processor.process_local_assign_statement(statement);

        statement.mutate_values().iter_mut()
            .for_each(|value| Self::visit_expression(value, processor));
    }

    fn visit_local_function(statement: &mut LocalFunctionStatement, processor: &mut T) {
        processor.process_local_function_statement(statement);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_numeric_for(statement: &mut NumericForStatement, processor: &mut T) {
        processor.process_numeric_for_statement(statement);

        Self::visit_expression(statement.mutate_start(), processor);
        Self::visit_expression(statement.mutate_end(), processor);

        if let Some(step) = statement.mutate_step() {
            Self::visit_expression(step, processor);
        };

        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_repeat_statement(statement: &mut RepeatStatement, processor: &mut T) {
        processor.process_repeat_statement(statement);

        Self::visit_expression(statement.mutate_condition(), processor);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_while_statement(statement: &mut WhileStatement, processor: &mut T) {
        processor.process_while_statement(statement);

        Self::visit_expression(statement.mutate_condition(), processor);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_field_expression(field: &mut FieldExpression, processor: &mut T) {
        processor.process_field_expression(field);

        Self::visit_prefix_expression(field.mutate_prefix(), processor);
    }

    fn visit_index_expression(index: &mut IndexExpression, processor: &mut T) {
        processor.process_index_expression(index);

        Self::visit_prefix_expression(index.mutate_prefix(), processor);
        Self::visit_expression(index.mutate_index(), processor);
    }

    fn visit_function_call(call: &mut FunctionCall, processor: &mut T) {
        processor.process_function_call(call);

        Self::visit_prefix_expression(call.mutate_prefix(), processor);
        Self::visit_arguments(call.mutate_arguments(), processor);
    }

    fn visit_arguments(arguments: &mut Arguments, processor: &mut T) {
        match arguments {
            Arguments::String(string) => processor.process_string_expression(string),
            Arguments::Table(table) => Self::visit_table(table, processor),
            Arguments::Tuple(expressions) => expressions.iter_mut()
                .for_each(|expression| Self::visit_expression(expression, processor)),
        }
    }

    fn visit_table(table: &mut TableExpression, processor: &mut T) {
        processor.process_table_expression(table);

        table.mutate_entries().iter_mut()
            .for_each(|entry| match entry {
                TableEntry::Field(_field, value) => Self::visit_expression(value, processor),
                TableEntry::Index(key, value) => {
                    Self::visit_expression(key, processor);
                    Self::visit_expression(value, processor);
                }
                TableEntry::Value(value) => Self::visit_expression(value, processor),
            });
    }

    fn visit_prefix_expression(prefix: &mut Prefix, processor: &mut T) {
        processor.process_prefix_expression(prefix);

        match prefix {
            Prefix::Call(call) => Self::visit_function_call(call, processor),
            Prefix::Field(field) => Self::visit_field_expression(field, processor),
            Prefix::Identifier(identifier) => processor.process_variable_expression(identifier),
            Prefix::Index(index) => Self::visit_index_expression(index, processor),
            Prefix::Parenthese(expression) => Self::visit_expression(expression, processor),
        };
    }
}

/// The default node visitor.
pub struct DefaultVisitor<T> {
    _phantom: PhantomData<T>,
}

impl<T: NodeProcessor> NodeVisitor<T> for DefaultVisitor<T> {}

/// The default mutable node visitor.
pub struct DefaultVisitorMut<T> {
    _phantom: PhantomData<T>,
}

impl<T: NodeProcessorMut> NodeVisitorMut<T> for DefaultVisitorMut<T> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::process::NodeCounter;

    #[test]
    fn visit_do_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(DoStatement::default());

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.do_count, 1);
    }

    #[test]
    fn visit_numeric_for_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(NumericForStatement::new(
                "i".to_owned(),
                Expression::True,
                Expression::True,
                None,
                Block::default(),
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 2);
        assert_eq!(counter.numeric_for_count, 1);
    }

    #[test]
    fn visit_generic_for_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(GenericForStatement::new(
                vec!["k".to_owned()],
                vec![Expression::True],
                Block::default(),
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.generic_for_count, 1);
    }

    #[test]
    fn visit_repeat_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(RepeatStatement::new(
                Block::default(),
                Expression::True,
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.repeat_count, 1);
    }

    #[test]
    fn visit_while_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(WhileStatement::new(
                Block::default(),
                Expression::True,
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.while_count, 1);
    }

    #[test]
    fn visit_if_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(IfStatement::create(
                Expression::True,
                Block::default(),
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.if_count, 1);
    }

    #[test]
    fn visit_if_statement_with_else() {
        let mut counter = NodeCounter::new();
        let if_statement = IfStatement::create(Expression::True, Block::default())
            .with_else_block(Block::default());

        let mut block = Block::default().with_statement(if_statement);

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 3);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.if_count, 1);
    }

    #[test]
    fn visit_if_statement_with_elseif_and_else() {
        let mut counter = NodeCounter::new();
        let if_statement = IfStatement::create(Expression::True, Block::default())
            .with_branch(Expression::False, Block::default())
            .with_else_block(Block::default());

        let mut block = Block::default().with_statement(if_statement);

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 4);
        assert_eq!(counter.expression_count, 2);
        assert_eq!(counter.if_count, 1);
    }
}
