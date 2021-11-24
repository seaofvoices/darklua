use darklua_core::rules::{RemoveComments, Rule};

macro_rules! test_remove_comments_rule {
    ($rule:expr, $($name:ident ($input:literal) => $output:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                use darklua_core::{
                    Parser,
                    generator::{LuaGenerator, TokenBasedLuaGenerator},
                    rules::Rule,
                };
                let mut context = $crate::rule_tests::get_process_context();

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

                pretty_assertions::assert_eq!(
                    $output,
                    lua_code,
                    "\nexpected code:\n{}\nbut received:\n{}\n",
                    $output,
                    lua_code,
                );
            }
        )*
    };
}

test_remove_comments_rule!(
    RemoveComments::default(),
    empty_do("do end -- comment") => "do end ",
    comment_after_semicolon("print('hello');-- bye") => "print('hello');",
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_comments',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_comments'").unwrap();
}
