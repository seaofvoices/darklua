use darklua_core::rules::{RemoveIfExpression, Rule};

test_rule!(
    remove_if_expression,
    RemoveIfExpression::default(),
    assign_if_expression("local a = if true then 1 else 2") => "local a = (function() if true then return 1 else return 2 end end)()",
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
