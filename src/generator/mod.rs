//! A module that contains the main [LuaGenerator](trait.LuaGenerator.html) trait
//! and its implementations.

mod dense;
mod readable;
mod token_based;
mod utils;

pub use dense::DenseLuaGenerator;
pub use readable::ReadableLuaGenerator;
pub use token_based::TokenBasedLuaGenerator;

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
            CompoundAssign(statement) => self.write_compound_assign(statement),
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
    fn write_compound_assign(&mut self, assign: &nodes::CompoundAssignStatement);
    fn write_generic_for(&mut self, generic_for: &nodes::GenericForStatement);
    fn write_if_statement(&mut self, if_statement: &nodes::IfStatement);
    fn write_function_statement(&mut self, function: &nodes::FunctionStatement);
    fn write_last_statement(&mut self, statement: &nodes::LastStatement);
    fn write_local_assign(&mut self, assign: &nodes::LocalAssignStatement);
    fn write_local_function(&mut self, function: &nodes::LocalFunctionStatement);
    fn write_numeric_for(&mut self, numeric_for: &nodes::NumericForStatement);
    fn write_repeat_statement(&mut self, repeat: &nodes::RepeatStatement);
    fn write_while_statement(&mut self, while_statement: &nodes::WhileStatement);

    fn write_variable(&mut self, variable: &nodes::Variable) {
        use nodes::Variable::*;
        match variable {
            Identifier(identifier) => self.write_identifier(identifier),
            Field(field) => self.write_field(field),
            Index(index) => self.write_index(index),
        }
    }

    fn write_expression(&mut self, expression: &nodes::Expression);

    fn write_identifier(&mut self, identifier: &nodes::Identifier);
    fn write_binary_expression(&mut self, binary: &nodes::BinaryExpression);
    fn write_if_expression(&mut self, if_expression: &nodes::IfExpression);
    fn write_unary_expression(&mut self, unary: &nodes::UnaryExpression);
    fn write_function(&mut self, function: &nodes::FunctionExpression);
    fn write_function_call(&mut self, call: &nodes::FunctionCall);
    fn write_field(&mut self, field: &nodes::FieldExpression);
    fn write_index(&mut self, index: &nodes::IndexExpression);
    fn write_parenthese(&mut self, parenthese: &nodes::ParentheseExpression);

    fn write_prefix(&mut self, prefix: &nodes::Prefix) {
        use nodes::Prefix::*;
        match prefix {
            Call(call) => self.write_function_call(call),
            Field(field) => self.write_field(field),
            Identifier(identifier) => self.write_identifier(identifier),
            Index(index) => self.write_index(index),
            Parenthese(parenthese) => self.write_parenthese(parenthese),
        }
    }

    fn write_table(&mut self, table: &nodes::TableExpression);
    fn write_table_entry(&mut self, entry: &nodes::TableEntry);
    fn write_number(&mut self, number: &nodes::NumberExpression);

    fn write_arguments(&mut self, arguments: &nodes::Arguments) {
        use nodes::Arguments::*;
        match arguments {
            String(string) => self.write_string(string),
            Table(table) => self.write_table(table),
            Tuple(tuple) => self.write_tuple_arguments(tuple),
        }
    }

    fn write_tuple_arguments(&mut self, arguments: &nodes::TupleArguments);

    fn write_string(&mut self, string: &nodes::StringExpression);
    fn write_interpolated_string(&mut self, string: &nodes::InterpolatedStringExpression);
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
                    use std::str::FromStr;
                    let number = $crate::nodes::NumberExpression::from_str($value).unwrap();

                    let mut generator = $generator;
                    generator.write_expression(&number.into());

                    assert_eq!(generator.into_string(), $value);
                }
            )*
        };
    }

    macro_rules! blocks_consistency {
        (
            $generator:expr => (
                $($name:ident => $code:literal),+,
            )
        ) => {
            $(
                #[test]
                fn $name() {
                    let parser = $crate::Parser::default();

                    let expected_block = parser.parse($code)
                        .expect(&format!("unable to parse `{}`", $code));

                    let mut generator = $generator;
                    generator.write_block(&expected_block);
                    let generated_code = generator.into_string();

                    let generated_block = parser.parse(&generated_code)
                        .expect(&format!("unable to parse generated code `{}`", &generated_code));

                    assert_eq!(expected_block, generated_block);
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
                        LastStatement::Return(statement) => statement.iter_expressions()
                            .next()
                            .unwrap(),
                        _ => panic!("return statement expected"),
                    };

                    let mut generator = $generator;
                    generator.write_expression(&$input.into());

                    let generated_code = format!("return {}", generator.into_string());
                    let parsed_block = parser.parse(&generated_code)
                        .expect(&format!("unable to parse generated code: `{}`", &generated_code));

                    let parsed_return = parsed_block.get_last_statement()
                        .expect("it should have a return statement");

                    let parsed = match parsed_return {
                        LastStatement::Return(statement) => {
                            if statement.len() != 1 {
                                panic!("return statement has more than one expression")
                            }
                            statement.iter_expressions().next().unwrap()
                        },
                        _ => panic!("return statement expected"),
                    };

                    pretty_assertions::assert_eq!(parsed, expected);
                }
            )*
        };
    }

    macro_rules! snapshot_generator {
        ($mod_name:ident, $generator:expr) => {

mod $mod_name {
    use super::*;
    use $crate::nodes::*;

    mod edge_cases {
        use super::*;

        blocks_consistency!($generator => (
            index_with_bracket_string => "return ok[ [[field]]]",
            call_with_bracket_string => "return ok[[ [field] ]]",
            concat_numbers => "return 9 .. 3",
            concat_float_numbers => "return 9. .. 3",
            concat_number_with_variable_arguments => "return 9 .. ...",
            concat_variable_arguments_with_number => "return ... ..1",
            double_unary_minus => "return - -10",
            binary_minus_with_unary_minus => "return 100- -10",
        ));
    }

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
                    false,
                    BinaryExpression::new(
                        BinaryOperator::Or,
                        false,
                        true,
                    ),
                )
            ) => "false and (false or true)",
            left_associative_wraps_right_operand_if_has_same_precedence(
                BinaryExpression::new(
                    BinaryOperator::Equal,
                    true,
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
            if_does_not_wrap_else(
                IfExpression::new(
                    Expression::identifier("condition"),
                    10.0,
                    BinaryExpression::new(
                        BinaryOperator::Percent,
                        9.0,
                        2.0,
                    ),
                )

            ) => "if condition then 10 else 9 % 2",
            binary_expression_wraps_if(
                BinaryExpression::new(
                    BinaryOperator::Percent,
                    IfExpression::new(Expression::identifier("condition"), 10.0, 9.0),
                    2.0,
                )
            ) => "(if condition then 10 else 9) % 2",
            unary_does_not_wrap_if_with_binary_in_else_result(
                UnaryExpression::new(
                    UnaryOperator::Not,
                    IfExpression::new(
                        Expression::identifier("condition"),
                        true,
                        BinaryExpression::new(
                            BinaryOperator::And,
                            false,
                            StringExpression::from_value("ok"),
                        )
                    ),
                )
            ) => "not if condition then true else false and 'ok'",
            binary_wraps_unary_containing_an_if_expression(
                BinaryExpression::new(
                    BinaryOperator::And,
                    UnaryExpression::new(
                        UnaryOperator::Not,
                        IfExpression::new(Expression::identifier("condition"), true, false),
                    ),
                    StringExpression::from_value("ok"),
                )
            ) => "(not if condition then true else false) and 'ok'",
        ));
    }

    mod snapshots {
        use super::*;

        snapshot_node!($mod_name, $generator, block, write_block => (
            ambiguous_function_call_from_assign => Block::default()
                .with_statement(
                    AssignStatement::from_variable(Variable::new("name"), Expression::identifier("variable"))
                )
                .with_statement(
                    AssignStatement::from_variable(
                        FieldExpression::new(ParentheseExpression::new(Expression::identifier("t")), "field"),
                        false
                    )
                ),
            ambiguous_function_call_from_compound_assign => Block::default()
                .with_statement(
                    CompoundAssignStatement::new(
                        CompoundOperator::Plus,
                        Variable::new("name"),
                        BinaryExpression::new(
                            BinaryOperator::Plus,
                            Expression::identifier("variable"),
                            Expression::identifier("value"),
                        )
                    )
                )
                .with_statement(
                    AssignStatement::from_variable(
                        IndexExpression::new(
                            ParentheseExpression::new(Expression::identifier("t")),
                            Expression::identifier("field"),
                        ),
                        false
                    )
                ),
            ambiguous_function_call_from_local_assign => Block::default()
                .with_statement(
                    LocalAssignStatement::from_variable("name")
                        .with_value(
                            IfExpression::new(
                                Expression::identifier("condition"),
                                true,
                                FunctionCall::from_name("fn")
                            )
                        )
                )
                .with_statement(
                    FunctionCall::from_prefix(ParentheseExpression::new(Expression::identifier("fn")))
                ),
            ambiguous_function_call_from_function_call => Block::default()
                .with_statement(
                    FunctionCall::from_name("fn")
                )
                .with_statement(
                    CompoundAssignStatement::new(
                        CompoundOperator::Plus,
                        IndexExpression::new(ParentheseExpression::new(
                            Expression::identifier("t")),
                            Expression::identifier("field"),
                        ),
                        1
                    )
                ),
            ambiguous_function_call_from_repeat => Block::default()
                .with_statement(
                    RepeatStatement::new(
                        Block::default(),
                        UnaryExpression::new(UnaryOperator::Not, Expression::identifier("variable"))
                    )
                )
                .with_statement(
                    CompoundAssignStatement::new(
                        CompoundOperator::Plus,
                        FieldExpression::new(ParentheseExpression::new(Expression::identifier("t")), "field"),
                        1
                    )
                ),
        ));

        snapshot_node!($mod_name, $generator, expression, write_expression => (
            false_value => false,
            true_value => true,
            nil_value => Expression::nil(),
            variable_arguments => Expression::variable_arguments(),
            true_in_parenthese => Expression::from(true).in_parentheses(),
        ));

        snapshot_node!($mod_name, $generator, assign, write_statement => (
            variable_with_one_value => AssignStatement::new(
                vec![Variable::new("var")],
                vec![Expression::from(false)],
            ),
            two_variables_with_one_value => AssignStatement::new(
                vec![Variable::new("foo"), Variable::new("var")],
                vec![Expression::from(false)],
            ),
            two_variables_with_two_values => AssignStatement::new(
                vec![Variable::new("foo"), Variable::new("var")],
                vec![Expression::nil(), Expression::from(false)],
            ),
        ));

        snapshot_node!($mod_name, $generator, do_statement, write_statement => (
            empty => DoStatement::default(),
            nested_do => DoStatement::new(
                Block::default().with_statement(DoStatement::default())
            ),
        ));

        snapshot_node!($mod_name, $generator, compound_assign_statement, write_statement => (
            increment_var_by_one => CompoundAssignStatement::new(
                CompoundOperator::Plus,
                Variable::new("var"),
                1_f64,
            ),
        ));

        snapshot_node!($mod_name, $generator, function_statement, write_statement => (
            empty => FunctionStatement::from_name("foo", Block::default()),
            empty_with_field =>  FunctionStatement::new(
                FunctionName::from_name("foo").with_fields(vec!["bar".into()]),
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
                vec!["var".into()],
                vec![Expression::from(true)],
                Block::default()
            ),
        ));

        snapshot_node!($mod_name, $generator, if_statement, write_statement => (
            empty => IfStatement::create(false, Block::default()),
            empty_with_empty_else => IfStatement::create(false, Block::default())
                .with_else_block(Block::default()),
            empty_with_empty_multiple_branch => IfStatement::create(false, Block::default())
                .with_new_branch(Expression::nil(), Block::default())
                .with_new_branch(false, Block::default()),
        ));

        snapshot_node!($mod_name, $generator, local_assign, write_statement => (
            foo_unassigned => LocalAssignStatement::from_variable("foo"),
            foo_and_bar_unassigned => LocalAssignStatement::from_variable("foo")
                .with_variable("bar"),
            var_assign_to_false => LocalAssignStatement::from_variable("var")
                .with_value(false),
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
                Expression::identifier("start"),
                Expression::identifier("max"),
                None,
                Block::default()
            ),
            empty_with_step => NumericForStatement::new(
                "i",
                Expression::identifier("start"),
                Expression::identifier("max"),
                Some(Expression::identifier("step")),
                Block::default()
            ),
        ));

        snapshot_node!($mod_name, $generator, repeat, write_statement => (
            empty => RepeatStatement::new(
                Block::default(),
                false
            ),
        ));

        snapshot_node!($mod_name, $generator, while_statement, write_statement => (
            empty => WhileStatement::new(
                Block::default(),
                false
            ),
        ));

        snapshot_node!($mod_name, $generator, last, write_last_statement => (
            break_statement => LastStatement::new_break(),
            continue_statement => LastStatement::new_continue(),
            return_without_values => ReturnStatement::default(),
            return_one_expression => ReturnStatement::one(Expression::from(true)),
            return_two_expressions => ReturnStatement::one(Expression::from(true))
                .with_expression(Expression::nil()),
            return_parentheses => ReturnStatement::one(Expression::from(true).in_parentheses()),
        ));

        snapshot_node!($mod_name, $generator, binary, write_expression => (
            true_and_false => BinaryExpression::new(
                BinaryOperator::And,
                Expression::from(true),
                Expression::from(false)
            ),
            true_equal_false =>BinaryExpression::new(
                BinaryOperator::Equal,
                Expression::from(true),
                Expression::from(false)
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
            identifier_in_parenthese => Prefix::Parenthese(ParentheseExpression::new(Expression::identifier("foo"))),
        ));

        snapshot_node!($mod_name, $generator, string, write_expression => (
            only_letters => StringExpression::from_value("hello"),
            with_single_quotes => StringExpression::from_value("I'm cool"),
            with_double_quotes => StringExpression::from_value(r#"Say: "Hi""#),
            with_single_and_double_quotes => StringExpression::from_value(r#"Say: "Don't""#),
        ));

        snapshot_node!($mod_name, $generator, interpolated_string, write_expression => (
            only_letters => InterpolatedStringExpression::empty()
                .with_segment("hello"),
            with_single_quotes => InterpolatedStringExpression::empty()
                .with_segment("I'm cool"),
            with_double_quotes => InterpolatedStringExpression::empty()
                .with_segment(r#"Say: "Hi""#),
            with_backticks => InterpolatedStringExpression::empty()
                .with_segment("Say: `Hi`"),
            with_single_and_double_quotes => InterpolatedStringExpression::empty()
                .with_segment(r#"Say: "Don't""#),
            with_true_value => InterpolatedStringExpression::empty()
                .with_segment(Expression::from(true)),
            with_empty_table => InterpolatedStringExpression::empty()
                .with_segment(Expression::from(TableExpression::default())),
        ));

        snapshot_node!($mod_name, $generator, number, write_expression => (
            number_1 => 1.0,
            number_0_5 => 0.5,
            number_123 => 123.0,
            number_0_005 => 0.005,
            number_nan => DecimalNumber::new(f64::NAN),
            number_positive_infinity => DecimalNumber::new(f64::INFINITY),
            number_negative_infinity => DecimalNumber::new(f64::NEG_INFINITY),
            number_1_2345e_minus50 => 1.2345e-50,
            number_thousand => 1000.0,
            number_1_2345e50 => 1.2345e50,
            number_100_25 => 100.25,
            number_2000_05 => 2000.05,
            binary_0b10101 => BinaryNumber::new(0b10101, false),
        ));

        snapshot_node!($mod_name, $generator, table, write_expression => (
            empty => TableExpression::default(),
            list_with_single_value => TableExpression::new(vec![
                TableEntry::Value(Expression::from(true)),
            ]),
            list_with_two_values => TableExpression::new(vec![
                TableEntry::Value(Expression::from(true)),
                TableEntry::Value(Expression::from(false)),
            ]),
            with_field_entry => TableExpression::new(vec![
                TableFieldEntry::new("field", true).into(),
            ]),
            with_index_entry => TableExpression::new(vec![
                TableIndexEntry::new(false, true).into(),
            ]),
            mixed_table => TableExpression::new(vec![
                TableEntry::Value(Expression::from(true)),
                TableFieldEntry::new("field", true).into(),
                TableIndexEntry::new(false, true).into(),
            ]),
        ));

        snapshot_node!($mod_name, $generator, unary, write_expression => (
            not_true => UnaryExpression::new(
                UnaryOperator::Not,
                true,
            ),
            two_unary_minus_breaks_between_them => UnaryExpression::new(
                UnaryOperator::Minus,
                UnaryExpression::new(
                    UnaryOperator::Minus,
                    Expression::identifier("a"),
                ),
            ),
            wraps_in_parens_if_an_inner_binary_has_lower_precedence => UnaryExpression::new(
                UnaryOperator::Not,
                BinaryExpression::new(
                    BinaryOperator::Or,
                    false,
                    true,
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
            empty_tuple => TupleArguments::default(),
            tuple_with_one_value => TupleArguments::new(vec![true.into()]),
            tuple_with_two_values => TupleArguments::new(vec![true.into(), false.into()]),
        ));
    }
}

        };
    }

    snapshot_generator!(dense, DenseLuaGenerator::default());
    snapshot_generator!(readable, ReadableLuaGenerator::default());
    snapshot_generator!(token_based, TokenBasedLuaGenerator::new(""));
}
