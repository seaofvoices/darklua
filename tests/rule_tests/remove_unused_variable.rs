use darklua_core::rules::{RemoveUnusedVariable, Rule};

test_rule!(
    RemoveUnusedVariable::default(),
    remove_unused_local("local foo = true return true") => "return true",
    remove_two_unused_local("local foo, bar = true return true") => "return true",
    remove_unused_local_function("local function foo() end return true") => "return true",
    remove_unused_local_function_single_statement("local function foo() end") => "",
    remove_unused_local_function_recursive("local function foo() foo() end return true") => "return true",
    remove_function_used_by_unused_function("local function foo() end local function bar() foo() end") => "",
    remove_local_but_keep_function_call("local foo = print('hello')") => "print('hello')",
    keep_only_used_constants("local a, b = true, false return b") => "local b = false return b",
    remove_unused_after_last_used_in_tuple_extract("local a, b, c = ... return b") => "local a, b = ... return b",
    remove_variable_before_tuple_extract("local a, b, c = true, ... return b") => "local b = ... return b",
    remove_variable_before_tuple_extract_and_after_last_used("local a, b, c = true, ... return c")
        => "local b, c = ... return c",
    keep_variable_before_tuple_extract_and_remove_after_last_used("local a, b, c, d = true, ... return a and c")
        => "local a, b, c = true, ... return a and c"
);

test_rule_wihout_effects!(
    RemoveUnusedVariable::default(),
    keep_returning_local_function("local function foo() end return foo"),
    keep_used_local_function("local function foo() end foo()"),
    keep_not_initialized_variable("local foo return foo"),
    keep_previous_identifiers_for_tuple_extraction("local a, b, c = ... return c")
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'remove_unused_variable',
    }"#).unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_unused_variable'").unwrap();
}
