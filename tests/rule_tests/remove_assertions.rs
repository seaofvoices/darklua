use darklua_core::rules::{RemoveAssertions, Rule};

test_rule!(
    remove_debug_profiling,
    RemoveAssertions::default(),
    remove_variable_condition("assert(condition)") => "do end",
    remove_variable_condition_with_message("assert(condition, 'message')") => "do end",
    remove_function_call_condition("assert(validate(value))") => "validate(value)",
    remove_variable_condition_with_function_call_message("assert(condition, formatter(condition))") => "formatter(condition)",
    remove_function_call_condition_and_function_call_message("assert(validate(value), formatter(value))") => "do validate(value) formatter(value) end",
);

test_rule!(
    remove_debug_profiling_without_side_effects,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_assertions',
        preserve_arguments_side_effects: false,
    }"#,
    )
    .unwrap(),
    remove_variable_condition("assert(condition)") => "do end",
    remove_variable_condition_with_message("assert(condition, 'message')") => "do end",
    remove_function_call_condition("assert(validate(value))") => "do end",
    remove_variable_condition_with_function_call_message("assert(condition, formatter(condition))") => "do end",
);

test_rule_without_effects!(
    RemoveAssertions::default(),
    assert_function_used("local function assert() end assert('label')"),
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_assertions',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_assertions'").unwrap();
}
