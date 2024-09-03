use darklua_core::rules::{RemoveContinue, Rule};

test_rule!(
    remove_continue,
    RemoveContinue::default(),
    continue_inside_numeric_for("for i = 1, 10 do continue end") => "for i = 1, 10 do repeat break until true end",
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_continue',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_continue'").unwrap();
}
