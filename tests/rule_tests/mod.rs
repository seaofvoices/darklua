macro_rules! test_rule {
    ($rule_name:ident, $rule:expr, $($name:ident ($input:literal) => $output:literal),* $(,)?) => {
        paste::paste! {

        mod [<$rule_name _with_readable_generator>] {
            use super::*;

        $(
            #[test]
            fn $name() {
                use darklua_core::generator::{LuaGenerator, ReadableLuaGenerator};

                let mut block = $crate::utils::parse_input($input);
                let expect_block = $crate::utils::parse_input($output);
                let mut context = darklua_core::rules::Context::default();

                $rule.process(&mut block, &mut context)
                    .expect("rule should suceed");

                let mut generator = ReadableLuaGenerator::default();
                generator.write_block(&block);
                let lua_code = generator.into_string();

                pretty_assertions::assert_eq!(
                    block,
                    expect_block,
                    "\nexpected code:\n{}\nbut received:\n{}",
                    $output,
                    lua_code
                );
            }
        )*

        }

        mod [<$rule_name _with_dense_generator>] {
            use super::*;

        $(
            #[test]
            fn $name() {
                use darklua_core::generator::{LuaGenerator, DenseLuaGenerator};

                let mut block = $crate::utils::parse_input($input);
                let expect_block = $crate::utils::parse_input($output);
                let mut context = darklua_core::rules::Context::default();

                $rule.process(&mut block, &mut context)
                    .expect("rule should suceed");

                let mut generator = DenseLuaGenerator::default();
                generator.write_block(&block);
                let lua_code = generator.into_string();

                pretty_assertions::assert_eq!(
                    block,
                    expect_block,
                    "\nexpected code:\n{}\nbut received:\n{}",
                    $output,
                    lua_code
                );
            }
        )*

        }

        mod [<$rule_name _with_token_based_generator>] {

        $(
            #[test]
            fn $name() {
                use super::*;
                use darklua_core::{
                    Parser,
                    generator::{LuaGenerator, TokenBasedLuaGenerator},
                };

                let expect_block = $crate::utils::parse_input($output);
                let mut context = darklua_core::rules::Context::default();

                let mut block = Parser::default()
                    .preserve_tokens()
                    .parse($input)
                    .unwrap_or_else(|error| {
                        panic!("could not parse content: {:?}\ncontent:\n{}", error, $input)
                    });

                $rule.process(&mut block, &mut context)
                    .expect("rule should suceed");

                let mut generator = TokenBasedLuaGenerator::new($input);
                generator.write_block(&block);
                let lua_code = generator.into_string();

                let compare_block = $crate::utils::parse_input(&lua_code);

                pretty_assertions::assert_eq!(
                    compare_block,
                    expect_block,
                    "\nexpected code:\n{}\nbut received:\n{}",
                    $output,
                    lua_code
                );
            }
        )*

        }
    }

    };
}

macro_rules! test_rule_wihout_effects {
    ($rule:expr, $($name:ident ($input:literal)),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                use darklua_core::rules::Rule;

                let mut block = $crate::utils::parse_input($input);
                let expect_block = block.clone();
                let mut context = darklua_core::rules::Context::default();

                $rule.process(&mut block, &mut context)
                    .expect("rule should suceed");

                pretty_assertions::assert_eq!(block, expect_block);
            }
        )*
    };
}

mod compute_expression;
mod convert_index_to_field;
mod group_local_assignment;
mod inject_value;
mod no_local_function;
mod remove_call_parens;
mod remove_comments;
mod remove_empty_do;
mod remove_method_definition;
mod remove_unused_if_branch;
mod remove_unused_while;
mod rename_variables;
mod virtual_execution;
