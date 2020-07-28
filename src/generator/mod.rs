//! A module that contains the main [LuaGenerator](trait.LuaGenerator.html) trait
//! and its implementations.

mod dense;
mod readable;
mod utils;

pub use dense::DenseLuaGenerator;
pub use readable::ReadableLuaGenerator;

use crate::nodes;

/// A trait to let its implementation define how the Lua code is generated. See
/// [ReadableLuaGenerator](struct.ReadableLuaGenerator.html) and
/// [DenseLuaGenerator](struct.DenseLuaGenerator.html) for implementations.
pub trait LuaGenerator {
    /// Consumes the LuaGenerator and produce a String object.
    fn into_string(self) -> String;

    fn write_block(&mut self, block: &nodes::Block);

    fn write_statement(&mut self, statement: &nodes::Statement) {
        use nodes::Statement::*;
        match statement {
            Assign(statement) => self.write_assign_statement(statement),
            Do(statement) => self.write_do_statement(statement),
            Call(statement) => self.write_function_call(statement),
            Function(statement) => self.write_function_statement(statement),
            GenericFor(statement) => self.write_generic_for(statement),
            If(statement) => self.write_if_statement(statement),
            LocalAssign(statement) => self.write_local_assign(statement),
            LocalFunction(statement) => self.write_local_function(statement),
            NumericFor(statement) => self.write_numeric_for(statement),
            Repeat(statement) => self.write_repeat_statement(statement),
            While(statement) => self.write_while_statement(statement),
        }
    }

    fn write_assign_statement(&mut self, assign: &nodes::AssignStatement);
    fn write_do_statement(&mut self, do_statement: &nodes::DoStatement);
    fn write_generic_for(&mut self, generic_for: &nodes::GenericForStatement);
    fn write_if_statement(&mut self, if_statement: &nodes::IfStatement);
    fn write_function_statement(&mut self, function: &nodes::FunctionStatement);
    fn write_last_statement(&mut self, statement: &nodes::LastStatement);
    fn write_local_assign(&mut self, assign: &nodes::LocalAssignStatement);
    fn write_local_function(&mut self, function: &nodes::LocalFunctionStatement);
    fn write_numeric_for(&mut self, numeric_for: &nodes::NumericForStatement);
    fn write_repeat_statement(&mut self, repeat: &nodes::RepeatStatement);
    fn write_while_statement(&mut self, while_statement: &nodes::WhileStatement);

    fn write_expression(&mut self, expression: &nodes::Expression);

    fn write_binary_expression(&mut self, binary: &nodes::BinaryExpression);
    fn write_unary_expression(&mut self, unary: &nodes::UnaryExpression);
    fn write_function(&mut self, function: &nodes::FunctionExpression);
    fn write_function_call(&mut self, call: &nodes::FunctionCall);
    fn write_field(&mut self, field: &nodes::FieldExpression);
    fn write_index(&mut self, index: &nodes::IndexExpression);
    fn write_prefix(&mut self, prefix: &nodes::Prefix);
    fn write_table(&mut self, table: &nodes::TableExpression);
    fn write_table_entry(&mut self, entry: &nodes::TableEntry);
    fn write_number(&mut self, number: &nodes::NumberExpression);

    fn write_arguments(&mut self, arguments: &nodes::Arguments);

    fn write_string(&mut self, string: &nodes::StringExpression);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::generator::{DenseLuaGenerator, ReadableLuaGenerator};

    macro_rules! snapshot_node {
        (
            $generator_name:ident, $generator:expr, $node_name:ident, $write_name:ident => (
                $($test_name:ident => $item:expr),+,
            )
        ) => {
            mod $node_name {
                use super::*;

                $(
                    #[test]
                    fn $test_name() {
                        let statement = $item;

                        let mut generator = $generator;
                        generator.$write_name(&statement.into());

                        let snapshot_name = concat!(
                            stringify!($generator_name),
                            "_",
                            stringify!($node_name),
                            "_",
                            stringify!($test_name),
                        );

                        insta::assert_snapshot!(
                            snapshot_name,
                            generator.into_string()
                        );
                    }
                )*
            }
        }
    }

    macro_rules! test_numbers {
        (
            $generator:expr => (
                $($name:ident => $value:expr),+,
            )
        ) => {
            $(
                #[test]
                fn $name() {
                    let number = $crate::nodes::NumberExpression::from($value);

                    let mut generator = $generator;
                    generator.write_expression(&number.into());

                    assert_eq!(generator.into_string(), $value);
                }
            )*
        };
    }

    macro_rules! binary_precedence {
        (
            $generator:expr => (
                $($name:ident($input:expr) => $expected:literal),+,
            )
        ) => {
            $(
                #[test]
                fn $name() {
                    let parser = $crate::Parser::default();

                    let expected_block = parser.parse(&format!("return {}", $expected))
                        .unwrap();
                    let expected_return = expected_block.get_last_statement()
                        .expect("it should have a return statement");

                    let expected = match expected_return {
                        LastStatement::Return(expressions) => expressions.first().unwrap(),
                        _ => panic!("return statement expected"),
                    };

                    let mut generator = $generator;
                    generator.write_expression(&$input.into());

                    let parsed_block = parser.parse(&format!("return {}", generator.into_string()))
                        .unwrap();

                    let parsed_return = parsed_block.get_last_statement()
                        .expect("it should have a return statement");

                    let parsed = match parsed_return {
                        LastStatement::Return(expressions) => {
                            if expressions.len() != 1 {
                                panic!("return statement has more than one expression")
                            }
                            expressions.first().unwrap()
                        },
                        _ => panic!("return statement expected"),
                    };

                    assert_eq!(parsed, expected);
                }
            )*
        };
    }

    macro_rules! snapshot_generator {
        ($mod_name:ident, $generator:expr) => {

mod $mod_name {
    use super::*;
    use $crate::nodes::*;

    mod numbers {
        use super::*;

        test_numbers!($generator => (
            zero => "0",
            one => "1",
            integer => "123",
            hex_number => "0x12",
            hex_number_with_letter => "0x12a",
            hex_with_exponent => "0x12p4",
        ));
    }

    mod binary {
        use super::*;

        binary_precedence!($generator => (
            left_associative_wraps_left_operand_if_has_lower_precedence(
                BinaryExpression::new(
                    BinaryOperator::Asterisk,
                    DecimalNumber::new(2.0),
                    BinaryExpression::new(
                        BinaryOperator::Plus,
                        DecimalNumber::new(1.0),
                        DecimalNumber::new(3.0),
                    )
                )
            ) => "2 * (1 + 3)",
            left_associative_wraps_right_operand_if_has_lower_precedence(
                BinaryExpression::new(
                    BinaryOperator::And,
                    Expression::False,
                    BinaryExpression::new(
                        BinaryOperator::Or,
                        Expression::False,
                        Expression::True,
                    ),
                )
            ) => "false and (false or true)",
            left_associative_wraps_right_operand_if_has_same_precedence(
                BinaryExpression::new(
                    BinaryOperator::Equal,
                    Expression::True,
                    BinaryExpression::new(
                        BinaryOperator::LowerThan,
                        DecimalNumber::new(1.0),
                        DecimalNumber::new(2.0),
                    ),
                )
            ) => "true == (1 < 2)",
            right_associative_wrap_unary_left_operand_if_has_lower_precedence(
                BinaryExpression::new(
                    BinaryOperator::Caret,
                    UnaryExpression::new(
                        UnaryOperator::Minus,
                        DecimalNumber::new(2.0),
                    ),
                    DecimalNumber::new(2.0),
                )
            ) => "(-2) ^ 2",
            right_associative_wraps_left_operand_if_has_lower_precedence(
                BinaryExpression::new(
                    BinaryOperator::Caret,
                    BinaryExpression::new(
                        BinaryOperator::Plus,
                        DecimalNumber::new(1.0),
                        DecimalNumber::new(2.0),
                    ),
                    DecimalNumber::new(3.0),
                )
            ) => "(1 + 2) ^ 3",
            right_associative_wraps_left_operand_if_has_same_precedence(
                BinaryExpression::new(
                    BinaryOperator::Caret,
                    BinaryExpression::new(
                        BinaryOperator::Caret,
                        DecimalNumber::new(2.0),
                        DecimalNumber::new(2.0),
                    ),
                    DecimalNumber::new(3.0),
                )
            ) => "(2 ^ 2) ^ 3",
            right_associative_does_not_wrap_right_operand_if_unary(
                BinaryExpression::new(
                    BinaryOperator::Caret,
                    DecimalNumber::new(2.0),
                    UnaryExpression::new(
                        UnaryOperator::Minus,
                        DecimalNumber::new(2.0),
                    ),
                )
            ) => "2 ^ -2",
            right_associative_does_not_wrap_right_operand_if_has_same_precedence(
                BinaryExpression::new(
                    BinaryOperator::Caret,
                    DecimalNumber::new(2.0),
                    BinaryExpression::new(
                        BinaryOperator::Caret,
                        DecimalNumber::new(2.0),
                        DecimalNumber::new(3.0),
                    ),
                )
            ) => "2 ^ 2 ^ 3",
            right_associative_does_not_wrap_right_operand_if_has_higher_precedence(
                BinaryExpression::new(
                    BinaryOperator::Concat,
                    DecimalNumber::new(3.0),
                    BinaryExpression::new(
                        BinaryOperator::Plus,
                        DecimalNumber::new(9.0),
                        DecimalNumber::new(3.0),
                    ),
                )
            ) => "3 .. 9 + 3",
        ));
    }

    mod snapshots {
        use super::*;
        snapshot_node!($mod_name, $generator, expression, write_expression => (
            false_value => Expression::False,
            true_value => Expression::True,
            nil_value => Expression::Nil,
            variable_arguments => Expression::VariableArguments,
            true_in_parenthese => Expression::Parenthese(Box::new(Expression::True)),
        ));

        snapshot_node!($mod_name, $generator, assign, write_statement => (
            variable_with_one_value => AssignStatement::new(
                vec![Variable::new("var")],
                vec![Expression::False],
            ),
            two_variables_with_one_value => AssignStatement::new(
                vec![Variable::new("foo"), Variable::new("var")],
                vec![Expression::False],
            ),
            two_variables_with_two_values => AssignStatement::new(
                vec![Variable::new("foo"), Variable::new("var")],
                vec![Expression::Nil, Expression::False],
            ),
        ));

        snapshot_node!($mod_name, $generator, do_statement, write_statement => (
            empty => DoStatement::default(),
            nested_do => DoStatement::new(
                Block::default().with_statement(DoStatement::default())
            ),
        ));

        snapshot_node!($mod_name, $generator, function_statement, write_statement => (
            empty => FunctionStatement::from_name("foo", Block::default()),
            empty_with_field =>  FunctionStatement::new(
                FunctionName::from_name("foo").with_fields(vec!["bar".to_owned()]),
                Block::default(),
                Vec::new(),
                false
            ),
            empty_with_method => FunctionStatement::new(
                FunctionName::from_name("foo").with_method("bar"),
                Block::default(),
                Vec::new(),
                false
            ),
        ));

        snapshot_node!($mod_name, $generator, generic_for, write_statement => (
            empty => GenericForStatement::new(
                vec!["var".to_owned()],
                vec![Expression::True],
                Block::default()
            ),
        ));

        snapshot_node!($mod_name, $generator, if_statement, write_statement => (
            empty => IfStatement::create(Expression::False, Block::default()),
            empty_with_empty_else => IfStatement::create(Expression::False, Block::default())
                .with_else_block(Block::default()),
            empty_with_empty_multiple_branch => IfStatement::create(Expression::False, Block::default())
                .with_branch(Expression::Nil, Block::default())
                .with_branch(Expression::False, Block::default()),
        ));

        snapshot_node!($mod_name, $generator, local_assign, write_statement => (
            foo_unassigned => LocalAssignStatement::from_variable("foo"),
            foo_and_bar_unassigned => LocalAssignStatement::from_variable("foo")
                .with_variable("bar"),
            var_assign_to_false => LocalAssignStatement::from_variable("var")
                .with_value(Expression::False),
        ));

        snapshot_node!($mod_name, $generator, local_function, write_statement => (
            empty => LocalFunctionStatement::from_name("foo", Block::default()),
            empty_variadic => LocalFunctionStatement::from_name("foo", Block::default())
                .variadic(),
            empty_with_one_parameter => LocalFunctionStatement::from_name("foo", Block::default())
                .with_parameter("bar"),
            empty_with_two_parameters => LocalFunctionStatement::from_name("foo", Block::default())
                .with_parameter("bar")
                .with_parameter("baz"),
            empty_variadic_with_one_parameter => LocalFunctionStatement::from_name("foo", Block::default())
                .with_parameter("bar")
                .variadic(),
        ));

        snapshot_node!($mod_name, $generator, numeric_for, write_statement => (
            empty_without_step => NumericForStatement::new(
                "i",
                Expression::Identifier("start".to_owned()),
                Expression::Identifier("max".to_owned()),
                None,
                Block::default()
            ),
            empty_with_step => NumericForStatement::new(
                "i",
                Expression::Identifier("start".to_owned()),
                Expression::Identifier("max".to_owned()),
                Some(Expression::Identifier("step".to_owned())),
                Block::default()
            ),
        ));

        snapshot_node!($mod_name, $generator, repeat, write_statement => (
            empty => RepeatStatement::new(
                Block::default(),
                Expression::False
            ),
        ));

        snapshot_node!($mod_name, $generator, while_statement, write_statement => (
            empty => WhileStatement::new(
                Block::default(),
                Expression::False
            ),
        ));

        snapshot_node!($mod_name, $generator, last, write_last_statement => (
            break_statement => LastStatement::Break,
            return_without_values => LastStatement::Return(Vec::new()),
            return_one_expression => LastStatement::Return(vec![Expression::True]),
            return_two_expressions => LastStatement::Return(vec![
                Expression::True, Expression::Nil
            ]),
        ));

        snapshot_node!($mod_name, $generator, binary, write_expression => (
            true_and_false => BinaryExpression::new(
                BinaryOperator::And,
                Expression::True,
                Expression::False
            ),
            true_equal_false =>BinaryExpression::new(
                BinaryOperator::Equal,
                Expression::True,
                Expression::False
            ),
        ));

        snapshot_node!($mod_name, $generator, field, write_expression => (
            identifier_prefix => FieldExpression::new(Prefix::from_name("foo"), "bar"),
        ));

        snapshot_node!($mod_name, $generator, index, write_expression => (
            identifier_prefix_with_identifier_value => IndexExpression::new(
                Prefix::from_name("foo"),
                Prefix::from_name("bar"),
            ),
        ));

        snapshot_node!($mod_name, $generator, function_expr, write_expression => (
            empty => FunctionExpression::default(),
            empty_variadic => FunctionExpression::default().variadic(),
            empty_variadic_with_one_parameter => FunctionExpression::default()
                .with_parameter("a")
                .variadic(),
            empty_variadic_with_two_parameter => FunctionExpression::default()
                .with_parameter("a")
                .with_parameter("b")
                .variadic(),
            empty_with_two_parameter => FunctionExpression::default()
                .with_parameter("a")
                .with_parameter("b"),
        ));

        snapshot_node!($mod_name, $generator, prefix, write_prefix => (
            identifier => Prefix::from_name("foo"),
            identifier_in_parenthese => Prefix::Parenthese(Prefix::from_name("foo").into()),
        ));

        snapshot_node!($mod_name, $generator, string, write_expression => (
            only_letters => StringExpression::from_value("hello"),
            with_single_quotes => StringExpression::from_value("I'm cool"),
            with_dougle_quotes => StringExpression::from_value(r#"Say: "Hi""#),
            with_single_and_double_quotes => StringExpression::from_value(r#"Say: "Don't""#),
        ));

        snapshot_node!($mod_name, $generator, number, write_expression => (
            number_1 => 1.0,
            number_0_5 => 0.5,
            number_123 => 123.0,
            number_0_005 => 0.005,
            number_nan => DecimalNumber::new(0.0/0.0),
            number_positive_infinity => DecimalNumber::new(1.0/0.0),
            number_negative_infinity => DecimalNumber::new(-1.0/0.0),
            number_1_2345e_minus50 => 1.2345e-50,
            number_thousand => 1000.0,
            number_1_2345e50 => 1.2345e50,
            number_100_25 => 100.25,
            number_2000_05 => 2000.05,
        ));

        snapshot_node!($mod_name, $generator, table, write_expression => (
            empty => TableExpression::default(),
            list_with_single_value => TableExpression::new(vec![
                TableEntry::Value(Expression::True),
            ]),
            list_with_two_values => TableExpression::new(vec![
                TableEntry::Value(Expression::True),
                TableEntry::Value(Expression::False),
            ]),
            with_field_entry => TableExpression::new(vec![
                TableEntry::Field("field".to_owned(), Expression::True),
            ]),
            with_index_entry => TableExpression::new(vec![
                TableEntry::Index(Expression::False, Expression::True),
            ]),
            mixed_table => TableExpression::new(vec![
                TableEntry::Value(Expression::True),
                TableEntry::Field("field".to_owned(), Expression::True),
                TableEntry::Index(Expression::False, Expression::True),
            ]),
        ));

        snapshot_node!($mod_name, $generator, unary, write_expression => (
            not_true => UnaryExpression::new(
                UnaryOperator::Not,
                Expression::True,
            ),
            two_unary_minus_breaks_between_them => UnaryExpression::new(
                UnaryOperator::Minus,
                UnaryExpression::new(
                    UnaryOperator::Minus,
                    Expression::Identifier("a".to_owned()),
                ),
            ),
            wraps_in_parens_if_an_inner_binary_has_lower_precedence => UnaryExpression::new(
                UnaryOperator::Not,
                BinaryExpression::new(
                    BinaryOperator::Or,
                    Expression::False,
                    Expression::True,
                ),
            ),
            does_not_wrap_in_parens_if_an_inner_binary_has_higher_precedence => UnaryExpression::new(
                UnaryOperator::Minus,
                BinaryExpression::new(
                    BinaryOperator::Caret,
                    DecimalNumber::new(2.0),
                    DecimalNumber::new(2.0),
                ),
            ),
        ));

        snapshot_node!($mod_name, $generator, arguments, write_arguments => (
            empty_tuple => Arguments::Tuple(Vec::new()),
            tuple_with_one_value => Arguments::Tuple(vec![Expression::True]),
            tuple_with_two_values => Arguments::Tuple(vec![Expression::True, Expression::False]),
        ));
    }
}

        };
    }

    snapshot_generator!(dense, DenseLuaGenerator::default());
    snapshot_generator!(readable, ReadableLuaGenerator::default());
}
