macro_rules! test_rule {
    ($rule:expr, $($name:ident ($input:literal) => $output:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                use darklua_core::rules::Rule;
                use darklua_core::generator::{LuaGenerator, ReadableLuaGenerator};

                let mut block = $crate::utils::parse_input($input);
                let expect_block = $crate::utils::parse_input($output);

                $rule.process(&mut block);

                let mut generator = ReadableLuaGenerator::default();
                generator.write_block(&block);
                let lua_code = generator.into_string();

                assert_eq!(
                    block,
                    expect_block,
                    "\nexpected code:\n{}\nbut received:\n{}",
                    $output,
                    lua_code
                );
            }
        )*
    };
}

macro_rules! test_rule_wihout_effects {
    ($rule:expr, $($name:ident ($input:literal)),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                use darklua_core::rules::Rule;
                use darklua_core::generator::{LuaGenerator, ReadableLuaGenerator};

                let mut block = $crate::utils::parse_input($input);

                let mut generator = ReadableLuaGenerator::default();
                generator.write_block(&block);
                let input_code = generator.into_string();

                let expect_block = block.clone();

                $rule.process(&mut block);

                let mut generator = ReadableLuaGenerator::default();
                generator.write_block(&block);
                let output_code = generator.into_string();

                assert_eq!(
                    block,
                    expect_block,
                    "\nexpected code:\n{}\nbut received:\n{}",
                    input_code,
                    output_code
                );
            }
        )*
    };
}

mod compute_expression;
mod group_local_assignment;
mod inject_value;
mod no_local_function;
mod remove_call_parens;
mod remove_empty_do;
mod remove_method_definition;
mod remove_unused_if_branch;
mod remove_unused_variable;
mod remove_unused_while;
mod rename_variables;
