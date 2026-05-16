use darklua_core::rules::{ConvertConstToLocal, Rule};

test_rule!(
    convert_const_to_local,
    ConvertConstToLocal::default(),
    const_assignment("const value = true") => "local value = true",
    typed_const_assignment("const value: boolean = true") => "local value: boolean = true",
    multi_const_assignment("const foo, bar = true, false") => "local foo, bar = true, false",
    const_function("const function foo() end") => "local function foo() end",
    typed_const_function("const function foo(value: boolean): boolean return value end") => "local function foo(value: boolean): boolean return value end",
    shadowed_const_assignment("const value = true do const value = false end") => "local value = true do local value = false end"
);

test_rule_with_tokens!(
    convert_const_to_local_preserve_tokens,
    ConvertConstToLocal::default(),
    const_assignment_with_spacing("const  value = true") => "local  value = true",
    const_function_with_spacing("const  function foo() end") => "local  function foo() end"
);

test_rule_without_effects!(
    ConvertConstToLocal::default(),
    local_assignment_named_const("local const = true"),
    regular_local_assignment("local value = true"),
    regular_local_function("local function foo() end")
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'convert_const_to_local',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'convert_const_to_local'").unwrap();
}
