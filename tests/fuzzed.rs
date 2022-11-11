// this test file is for collecting fuzzed Lua code that were causing issues that have been fixed

macro_rules! generate_tests {
    ($( $name:ident =>  $file_name:literal ),* $(,)?) => {
        $(
            #[cfg(not(debug_assertions))]
            #[test]
            fn $name() {
                use darklua_core::Parser;

                if cfg!(debug_assertions) {
                    panic!("running test in debug mode :(")
                };

                let input = include_str!($file_name);

                match Parser::default().parse(&input) {
                    Ok(_block) => (),
                    Err(error) => panic!("could not parse content: {:?}\ncontent:\n{}", error, input),
                }

            }
        )*
    }
}

generate_tests!(
    ast_conversion_stackoverflow_in_debug => "./fuzzed_test_cases/a.lua",
);
