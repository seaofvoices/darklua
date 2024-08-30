use darklua_core::rules::{RemoveIfExpression, Rule};

test_rule!(
    remove_if_expression,
    RemoveIfExpression::default(),
    assign_if_expression("local a = if true then 1 else 2") => "local a = (true and {(1)} or {(2)})[1]",
    assign_if_expression_with_elseif("local a = if true then 1 elseif false then 2 else 3") => "local a = (true and {(1)} or {(false and {(2)} or {(3)})[1]})[1]",
    if_expression_with_varargs("local function f(...: string) return if condition(...) then ... else transform(...) end") => "local function f(...: string) return (condition(...) and {(...)} or {(transform(...))})[1] end",
    if_expression_with_varargs_elseif("local function f(...: string) return if condition(...) then ... elseif condition(...) then ... else transform(...) end") => "local function f(...: string) return (condition(...) and {(...)} or {(condition(...) and {(...)} or {(transform(...))})[1]})[1] end"
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_if_expression',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_if_expression'").unwrap();
}
