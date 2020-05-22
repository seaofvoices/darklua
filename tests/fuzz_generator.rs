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
    LuaGenerator,
    ToLua,
};
use std::time::{Duration, Instant};

mod fuzz;
mod utils;

use fuzz::*;

macro_rules! fuzz_test_expression {
    ($node:expr,  $column_span:expr) => {
        let node: Expression = $node.into();
        let mut generator = LuaGenerator::new($column_span);
        node.to_lua(&mut generator);
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
            generated_node.to_lua_string(),
        );
    };
    ($node:expr) => {
        fuzz_test_expression!($node, 80);
    };
}

macro_rules! fuzz_test_block {
    ($context:expr, $column_span:expr) => {
        let block = Block::fuzz(&mut $context);

        let mut generator = LuaGenerator::new($column_span);
        block.to_lua(&mut generator);
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
            generated_block.to_lua_string(),
        );
    };
    ($context:expr) => {
        fuzz_test_block!($context, 80);
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

#[test]
fn fuzz_three_terms_binary_expressions() {
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

        fuzz_test_expression!(binary);
    });
}

#[test]
fn fuzz_binary_expressions_with_one_unary_expression() {
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

        fuzz_test_expression!(binary);
    });
}

#[test]
fn fuzz_single_statement() {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(1, 5));
    });
}

#[test]
fn fuzz_small_block() {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(20, 40));
    });
}

#[test]
fn fuzz_medium_block() {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(100, 200));
    });
}

#[test]
fn fuzz_large_block() {
    run_for_minimum_time(|| {
        fuzz_test_block!(FuzzContext::new(200, 500));
    });
}

#[test]
fn fuzz_column_span() {
    run_for_minimum_time(|| {
        for i in 0..80 {
            fuzz_test_block!(FuzzContext::new(20, 40), i);
        }
    });
}
