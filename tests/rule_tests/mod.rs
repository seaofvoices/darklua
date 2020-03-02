macro_rules! test_rule {
    ($rule:expr, $($name:ident ($input:literal) => $output:literal),*) => {
        $(
            #[test]
            fn $name() {
                use darklua_core::rules::Rule;

                let mut block = $crate::utils::parse_input($input);
                let expect_block = $crate::utils::parse_input($output);

                $rule.process(&mut block);

                assert_eq!(block, expect_block);
            }
        )*
    };
}

mod remove_empty_do;
mod rename_variables;
