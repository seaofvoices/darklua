use darklua_core::{
    nodes::{
        BinaryExpression,
        BinaryOperator,
        Block,
        Expression,
        LastStatement,
        UnaryExpression,
        UnaryOperator,
    },
    generator::LuaGenerator,
};
use std::time::{Duration, Instant};

mod fuzz;
mod utils;

use fuzz::*;

macro_rules! fuzz_test_expression {
    ($node:expr,  $generator:expr) => {
        let node: Expression = $node.into();
        let mut generator = $generator;
        generator.write_expression(&node);
        let lua_code = format!("return {}", generator.into_string());

        let mut generated_block = match utils::try_parse_input(&lua_code) {
            Ok(block) => block,
            Err(error) => panic!(
                concat!(
                    "could not parse content: {:?}\n",
                    "============================================================\n",
                    ">>> Lua code input:\n{}\n",
                    "============================================================\n",
                    "\n",
                    "============================================================\n",
                    ">>> Node that produced the generated code:\n{:?}\n",
                    "============================================================\n",
                ),
                error,
                lua_code,
                node,
            ),
        };

        let last_statement = generated_block.mutate_last_statement()
            .take()
            .expect("should have a last statement");

        let generated_node = match last_statement {
            LastStatement::Return(expressions) => {
                if expressions.len() != 1 {
                    panic!("should have exactly one expression")
                }
                expressions.into_iter().next().unwrap()
            }
            _ => panic!("return statement expected"),
        };

        let mut compare_generator = darklua_core::generator::ReadableLuaGenerator::default();
        compare_generator.write_expression(&generated_node);
        let generated_lua_code = compare_generator.into_string();

        assert_eq!(
            node,
            generated_node,
            concat!(
                "\n",
                "============================================================\n",
                ">>> Generated from node fuzz:\n{:#?}\n",
                ">>> Lua code generated:\n{}\n",
                "============================================================\n",
                "\n",
                "============================================================\n",
                ">>> Parsed node:\n{:#?}\n",
                ">>> Node code generated:\n{}\n",
                "============================================================\n",
            ),
            node,
            lua_code,
            generated_node,
            generated_lua_code,
        );
    };
}

macro_rules! fuzz_test_block {
    ($context:expr, $generator:expr) => {
        let block = Block::fuzz(&mut $context);

        let mut generator = $generator;
        generator.write_block(&block);
        let lua_code = generator.into_string();

        let generated_block = match utils::try_parse_input(&lua_code) {
            Ok(block) => block,
            Err(error) => panic!(
                concat!(
                    "could not parse content: {:?}\n",
                    "============================================================\n",
                    ">>> Lua code input:\n{}\n",
                    "============================================================\n",
                    "\n",
                    "============================================================\n",
                    ">>> Block that produced the generated code:\n{:?}\n",
                    "============================================================\n",
                ),
                error,
                lua_code,
                block,
            ),
        };

        let mut compare_generator = darklua_core::generator::ReadableLuaGenerator::default();
        compare_generator.write_block(&generated_block);
        let generated_lua_code = compare_generator.into_string();

        assert_eq!(
            block,
            generated_block,
            concat!(
                "\n",
                "============================================================\n",
                ">>> Generated from block fuzz:\n{:?}\n",
                ">>> Lua code generated:\n{}\n",
                "============================================================\n",
                "\n",
                "============================================================\n",
                ">>> Parsed generated block:\n{:?}\n",
                ">>> Lua code generated:\n{}\n",
                "============================================================\n",
            ),
            block,
            lua_code,
            generated_block,
            generated_lua_code,
        );
    };
}

fn run_for_minimum_time<F: Fn()>(func: F) {
    let duration = get_fuzz_duration();
    let start = Instant::now();

    loop {
        func();

        if Instant::now().duration_since(start) > duration {
            break
        }
    }
}

fn get_fuzz_duration() -> Duration {
    let millis = option_env!("FUZZ_DURATION_MILLISECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1500);

    Duration::from_millis(millis)
}

fn fuzz_three_terms_binary_expressions<T: LuaGenerator + Clone>(generator: T) {
    run_for_minimum_time(|| {
        let mut empty_context = FuzzContext::new(0, 0);
        let first = Expression::True;
        let second = Expression::False;
        let third = Expression::Nil;

        let (left, right) = if rand::random() {
            (
                BinaryExpression::new(
                    BinaryOperator::fuzz(&mut empty_context),
                    first,
                    second,
                ).into(),
                third,
            )
        } else {
            (
                first,
                BinaryExpression::new(
                    BinaryOperator::fuzz(&mut empty_context),
                    second,
                    third,
                ).into(),
            )
        };

        let operator = BinaryOperator::fuzz(&mut empty_context);
        let binary = BinaryExpression::new(
            operator,
            if operator.left_needs_parentheses(&left) {
                Expression::Parenthese(left.into())
            } else {
                left
            },
            if operator.right_needs_parentheses(&right) {
                Expression::Parenthese(right.into())
            } else {
                right
            }
        );

        fuzz_test_expression!(binary, generator.clone());
    });
}

fn fuzz_binary_expressions_with_one_unary_expression<T: LuaGenerator + Clone>(generator: T) {
    run_for_minimum_time(|| {
        let mut empty_context = FuzzContext::new(0, 0);
        let first = Expression::True;
        let second = Expression::False;

        let (left, right) = if rand::random() {
            (
                UnaryExpression::new(UnaryOperator::fuzz(&mut empty_context), first).into(),
                second,
            )
        } else {
            (
                first,
                UnaryExpression::new(UnaryOperator::fuzz(&mut empty_context), second).into(),
            )
        };

        let operator = BinaryOperator::fuzz(&mut empty_context);
        let binary = BinaryExpression::new(
            operator,
            if operator.left_needs_parentheses(&left) {
                Expression::Parenthese(left.into())
            } else {
                left
            },
            if operator.right_needs_parentheses(&right) {
                Expression::Parenthese(right.into())
            } else {
                right
            }
        );

        fuzz_test_expression!(binary, generator.clone());
    });
}

fn fuzz_single_statement<T: LuaGenerator + Clone>(generator: T) {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(1, 5), generator.clone());
    });
}

fn fuzz_tiny_block<T: LuaGenerator + Clone>(generator: T) {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(2, 8), generator.clone());
    });
}

fn fuzz_small_block<T: LuaGenerator + Clone>(generator: T) {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(20, 40), generator.clone());
    });
}

fn fuzz_medium_block<T: LuaGenerator + Clone>(generator: T) {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(100, 200), generator.clone());
    });
}

fn fuzz_large_block<T: LuaGenerator + Clone>(generator: T) {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(200, 500), generator.clone());
    });
}

mod dense_generator {
    use darklua_core::{
        nodes::Block,
        generator::{LuaGenerator, DenseLuaGenerator},
    };
    use super::fuzz::*;
    use super::utils;

    fn generator() -> DenseLuaGenerator {
        DenseLuaGenerator::new(80)
    }

    #[test]
    fn fuzz_three_terms_binary_expressions() {
        super::fuzz_three_terms_binary_expressions(generator());
    }

    #[test]
    fn fuzz_binary_expressions_with_one_unary_expression() {
        super::fuzz_binary_expressions_with_one_unary_expression(generator());
    }

    #[test]
    fn fuzz_single_statement() {
        super::fuzz_single_statement(generator());
    }

    #[test]
    fn fuzz_tiny_block() {
        super::fuzz_tiny_block(generator());
    }

    #[test]
    fn fuzz_small_block() {
        super::fuzz_small_block(generator());
    }

    #[test]
    fn fuzz_medium_block() {
        super::fuzz_medium_block(generator());
    }

    #[test]
    fn fuzz_large_block() {
        super::fuzz_large_block(generator());
    }

    #[test]
    fn fuzz_column_span() {
        super::run_for_minimum_time(|| {
            for i in 0..80 {
                let generator = DenseLuaGenerator::new(i);
                fuzz_test_block!(FuzzContext::new(20, 40), generator);
            }
        });
    }
}

mod readable_generator {
    use darklua_core::{
        nodes::Block,
        generator::{LuaGenerator, ReadableLuaGenerator},
    };
    use super::fuzz::*;
    use super::utils;

    fn generator() -> ReadableLuaGenerator {
        ReadableLuaGenerator::new(80)
    }

    #[test]
    fn fuzz_three_terms_binary_expressions() {
        super::fuzz_three_terms_binary_expressions(generator());
    }

    #[test]
    fn fuzz_binary_expressions_with_one_unary_expression() {
        super::fuzz_binary_expressions_with_one_unary_expression(generator());
    }

    #[test]
    fn fuzz_single_statement() {
        super::fuzz_single_statement(generator());
    }

    #[test]
    fn fuzz_tiny_block() {
        super::fuzz_tiny_block(generator());
    }

    #[test]
    fn fuzz_small_block() {
        super::fuzz_small_block(generator());
    }

    #[test]
    fn fuzz_medium_block() {
        super::fuzz_medium_block(generator());
    }

    #[test]
    fn fuzz_large_block() {
        super::fuzz_large_block(generator());
    }

    #[test]
    fn fuzz_column_span() {
        super::run_for_minimum_time(|| {
            for i in 0..80 {
                let generator = ReadableLuaGenerator::new(i);
                fuzz_test_block!(FuzzContext::new(20, 40), generator);
            }
        });
    }
}
