pub(crate) use crate::utils::memory_resources;

macro_rules! test_rule_with_generator {
    (
        $rule:expr,
        $resources:expr,
        $generator:expr,
        $parser:expr,
        $compare_with_tokens:expr,
        $test_file_name:literal,
        $name:ident,
        $input:literal,
        $output:literal
    ) => {
        #[test]
        fn $name() {
            use darklua_core::generator::LuaGenerator;

            // $crate::utils::setup_logger(log::LevelFilter::Trace);

            let expect_block = if $compare_with_tokens {
                darklua_core::Parser::default()
                    .preserve_tokens()
                    .parse($output)
                    .expect("unable to parse expected code")
            } else {
                $crate::utils::parse_input($output)
            };

            let parser = $parser;
            let mut block = parser.parse($input).unwrap_or_else(|error| {
                panic!("could not parse content: {:?}\ncontent:\n{}", error, $input)
            });

            let resources = $resources;
            resources.write($test_file_name, $input).unwrap();

            let context =
                darklua_core::rules::ContextBuilder::new($test_file_name, &resources, $input)
                    .build();

            $rule
                .process(&mut block, &context)
                .expect("rule should succeed");

            let create_generator = $generator;
            let mut generator = create_generator($input);
            generator.write_block(&block);
            let lua_code = generator.into_string();

            if $compare_with_tokens {
                pretty_assertions::assert_eq!($output, lua_code,);
            } else {
                pretty_assertions::assert_eq!(
                    $crate::utils::parse_input(&lua_code),
                    expect_block,
                    "\nexpected code:\n{}\nbut received:\n{}",
                    $output,
                    lua_code
                );
            }
        }
    };
    ($rule:expr, $resources:expr, $generator:expr, $test_file_name:literal, $name:ident, $input:literal, $output:literal) => {
        test_rule_with_generator!(
            $rule,
            $resources,
            $generator,
            darklua_core::Parser::default(),
            false,
            $test_file_name,
            $name,
            $input,
            $output
        );
    };
}

macro_rules! test_rule_with_tokens {
    (
        $rule_name:ident,
        $rule:expr,
        resources = $resources:expr,
        test_file_name = $test_file_name:literal,
        $($name:ident ($input:literal) => $output:literal),* $(,)?
    ) => {
        paste::paste! {

        mod [<$rule_name _with_token_based_generator>] {
            use super::*;

        $(
            test_rule_with_generator!(
                $rule,
                $resources,
                |input| darklua_core::generator::TokenBasedLuaGenerator::new(input),
                darklua_core::Parser::default().preserve_tokens(),
                true,
                $test_file_name,
                $name,
                $input,
                $output
            );
        )*

        }

        }
    };

    (
        $rule_name:ident,
        $rule:expr,
        resources = $resources:expr,
        $($name:ident ($input:literal) => $output:literal),* $(,)?
    ) => {
        test_rule_with_tokens!(
            $rule_name,
            $rule,
            resources = $resources,
            test_file_name = "src/test.lua",
            $( $name ($input) => $output, )*
        );
    };

    (
        $rule_name:ident,
        $rule:expr,
        test_file_name = $test_file_name:literal,
        $($name:ident ($input:literal) => $output:literal),* $(,)?
    ) => {
        test_rule_with_tokens!(
            $rule_name,
            $rule,
            resources = darklua_core::Resources::from_memory(),
            test_file_name = $test_file_name,
            $( $name ($input) => $output, )*
        );
    };

    ($rule_name:ident, $rule:expr, $($name:ident ($input:literal) => $output:literal),* $(,)?) => {
        test_rule_with_tokens!(
            $rule_name,
            $rule,
            resources = darklua_core::Resources::from_memory(),
            test_file_name = "src/test.lua",
            $( $name ($input) => $output, )*
        );
    };
}

macro_rules! test_rule {
    (
        $rule_name:ident,
        $rule:expr,
        resources = $resources:expr,
        test_file_name = $test_file_name:literal,
        $($name:ident ($input:literal) => $output:literal),* $(,)?
    ) => {
        paste::paste! {

        mod [<$rule_name _with_readable_generator>] {
            use super::*;

        $(
            test_rule_with_generator!(
                $rule,
                $resources,
                |_| darklua_core::generator::ReadableLuaGenerator::default(),
                $test_file_name,
                $name,
                $input,
                $output
            );
        )*

        }

        mod [<$rule_name _with_dense_generator>] {
            use super::*;

        $(
            test_rule_with_generator!(
                $rule,
                $resources,
                |_| darklua_core::generator::DenseLuaGenerator::default(),
                $test_file_name,
                $name,
                $input,
                $output
            );
        )*

        }

        mod [<$rule_name _with_token_based_generator>] {
            use super::*;

        $(
            test_rule_with_generator!(
                $rule,
                $resources,
                |input| darklua_core::generator::TokenBasedLuaGenerator::new(input),
                darklua_core::Parser::default().preserve_tokens(),
                false,
                $test_file_name,
                $name,
                $input,
                $output
            );
        )*

        }
    }

    };

    (
        $rule_name:ident,
        $rule:expr,
        resources = $resources:expr,
        $($name:ident ($input:literal) => $output:literal),* $(,)?
    ) => {
        test_rule!(
            $rule_name,
            $rule,
            resources = $resources,
            test_file_name = "src/test.lua",
            $( $name ($input) => $output, )*
        );
    };

    (
        $rule_name:ident,
        $rule:expr,
        test_file_name = $test_file_name:literal,
        $($name:ident ($input:literal) => $output:literal),* $(,)?
    ) => {
        test_rule!(
            $rule_name,
            $rule,
            resources = darklua_core::Resources::from_memory(),
            test_file_name = $test_file_name,
            $( $name ($input) => $output, )*
        );
    };

    ($rule_name:ident, $rule:expr, $($name:ident ($input:literal) => $output:literal),* $(,)?) => {
        test_rule!(
            $rule_name,
            $rule,
            resources = darklua_core::Resources::from_memory(),
            test_file_name = "src/test.lua",
            $( $name ($input) => $output, )*
        );
    };
}

macro_rules! test_rule_without_effects {
    ($rule:expr, $($name:ident ($input:literal)),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                use darklua_core::{
                    rules::Rule,
                    generator::{LuaGenerator, TokenBasedLuaGenerator},
                };

                let mut block = $crate::utils::parse_input($input);
                let expect_block = block.clone();
                let resources = darklua_core::Resources::from_memory();
                let context = darklua_core::rules::ContextBuilder::new(".", &resources, $input).build();

                $rule.process(&mut block, &context)
                    .expect("rule should succeed");

                let mut generator = TokenBasedLuaGenerator::new($input);
                generator.write_block(&block);
                let lua_code = generator.into_string();

                pretty_assertions::assert_eq!(
                    block,
                    expect_block,
                    "\nexpected code:\n{}\nbut received:\n{}",
                    $input,
                    lua_code,
                );
            }
        )*
    };
}

mod append_text_comment;
mod compute_expression;
mod convert_index_to_field;
mod convert_require;
mod filter_early_return;
mod group_local_assignment;
mod inject_value;
mod no_local_function;
mod remove_assertions;
mod remove_call_parens;
mod remove_comments;
mod remove_compound_assignment;
mod remove_debug_profiling;
mod remove_empty_do;
mod remove_interpolated_string;
mod remove_method_definition;
mod remove_nil_declaration;
mod remove_types;
mod remove_unused_if_branch;
mod remove_unused_variable;
mod remove_unused_while;
mod rename_variables;
