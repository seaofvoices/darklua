use crate::nodes::*;
use crate::process::NodeProcessor;

/// A trait that defines method that iterates on nodes and process them using a NodeProcessor.
pub trait NodeVisitor {
    fn visit_block<T: NodeProcessor>(block: &mut Block, processor: &mut T) {
        processor.process_block(block);

        block.mutate_statements()
            .iter_mut()
            .for_each(|statement| Self::visit_statement(statement, processor));

        if let Some(last_statement) = block.mutate_last_statement() {
            processor.process_last_statement(last_statement);

            match last_statement {
                LastStatement::Return(expressions) => {
                    expressions.iter_mut()
                        .for_each(|expression| Self::visit_expression(expression, processor));
                }
                _ => {}
            };
        };
    }

    fn visit_statement<T: NodeProcessor>(statement: &mut Statement, processor: &mut T) {
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

    fn visit_expression<T: NodeProcessor>(expression: &mut Expression, processor: &mut T) {
        processor.process_expression(expression);

        match expression {
            Expression::Binary(expression) => {
                processor.process_binary_expression(expression);
                Self::visit_expression(expression.mutate_left(), processor);
                Self::visit_expression(expression.mutate_right(), processor);
            }
            Expression::Call(expression) => Self::visit_function_call(expression, processor),
            Expression::Field(field) => Self::visit_field_expression(field, processor),
            Expression::Function(function) => {
                processor.process_function_expression(function);

                Self::visit_block(function.mutate_block(), processor);
            }
            Expression::Identifier(identifier) => processor.process_identifier(identifier),
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

    fn visit_assign_statement<T: NodeProcessor>(statement: &mut AssignStatement, processor: &mut T) {
        processor.process_assign_statement(statement);

        statement.mutate_variables().iter_mut()
            .for_each(|variable| match variable {
                Variable::Identifier(identifier) => processor.process_identifier(identifier),
                Variable::Field(field) => Self::visit_field_expression(field, processor),
                Variable::Index(index) => Self::visit_index_expression(index, processor),
            });

        statement.mutate_values().iter_mut()
            .for_each(|expression| Self::visit_expression(expression, processor));
    }

    fn visit_do_statement<T: NodeProcessor>(statement: &mut DoStatement, processor: &mut T) {
        processor.process_do_statement(statement);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_function_statement<T: NodeProcessor>(statement: &mut FunctionStatement, processor: &mut T) {
        processor.process_function_statement(statement);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_generic_for<T: NodeProcessor>(statement: &mut GenericForStatement, processor: &mut T) {
        processor.process_generic_for_statement(statement);

        statement.mutate_expressions().iter_mut()
            .for_each(|expression| Self::visit_expression(expression, processor));
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_if_statement<T: NodeProcessor>(statement: &mut IfStatement, processor: &mut T) {
        processor.process_if_statement(statement);

        statement.mutate_branchs()
            .iter_mut()
            .for_each(|branch| {
                Self::visit_expression(branch.mutate_condition(), processor);
                Self::visit_block(branch.mutate_block(), processor);
            });

        if let Some(block) = statement.mutate_else_block() {
            Self::visit_block(block, processor);
        }
    }

    fn visit_local_assign<T: NodeProcessor>(statement: &mut LocalAssignStatement, processor: &mut T) {
        processor.process_local_assign_statement(statement);

        statement.mutate_values().iter_mut()
            .for_each(|value| Self::visit_expression(value, processor));
    }

    fn visit_local_function<T: NodeProcessor>(statement: &mut LocalFunctionStatement, processor: &mut T) {
        processor.process_local_function_statement(statement);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_numeric_for<T: NodeProcessor>(statement: &mut NumericForStatement, processor: &mut T) {
        processor.process_numeric_for_statement(statement);

        Self::visit_expression(statement.mutate_start(), processor);
        Self::visit_expression(statement.mutate_end(), processor);

        if let Some(step) = statement.mutate_step() {
            Self::visit_expression(step, processor);
        };

        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_repeat_statement<T: NodeProcessor>(statement: &mut RepeatStatement, processor: &mut T) {
        processor.process_repeat_statement(statement);

        Self::visit_expression(statement.mutate_condition(), processor);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_while_statement<T: NodeProcessor>(statement: &mut WhileStatement, processor: &mut T) {
        processor.process_while_statement(statement);

        Self::visit_expression(statement.mutate_condition(), processor);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_field_expression<T: NodeProcessor>(field: &mut FieldExpression, processor: &mut T) {
        processor.process_field_expression(field);

        Self::visit_prefix_expression(field.mutate_prefix(), processor);
    }

    fn visit_index_expression<T: NodeProcessor>(index: &mut IndexExpression, processor: &mut T) {
        processor.process_index_expression(index);

        Self::visit_prefix_expression(index.mutate_prefix(), processor);
    }

    fn visit_function_call<T: NodeProcessor>(call: &mut FunctionCall, processor: &mut T) {
        processor.process_function_call(call);

        Self::visit_prefix_expression(call.mutate_prefix(), processor);
        Self::visit_arguments(call.mutate_arguments(), processor);
    }

    fn visit_arguments<T: NodeProcessor>(arguments: &mut Arguments, processor: &mut T) {
        match arguments {
            Arguments::String(string) => processor.process_string_expression(string),
            Arguments::Table(table) => Self::visit_table(table, processor),
            Arguments::Tuple(expressions) => expressions.iter_mut()
                .for_each(|expression| processor.process_expression(expression)),
        }
    }

    fn visit_table<T: NodeProcessor>(table: &mut TableExpression, processor: &mut T) {
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

    fn visit_prefix_expression<T: NodeProcessor>(prefix: &mut Prefix, processor: &mut T) {
        processor.process_prefix_expression(prefix);

        match prefix {
            Prefix::Call(call) => Self::visit_function_call(call, processor),
            Prefix::Field(field) => Self::visit_field_expression(field, processor),
            Prefix::Identifier(_) => {},
            Prefix::Index(index) => Self::visit_index_expression(index, processor),
            Prefix::Parenthese(expression) => Self::visit_expression(expression, processor),
        };
    }
}

/// The default node visitor.
pub struct DefaultVisitor;

impl NodeVisitor for DefaultVisitor {}

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
