use darklua_core::rules::{RemoveNilDeclaration, Rule};

test_rule!(
    remove_nil_declaration,
    RemoveNilDeclaration::default(),
    assign_to_nil("local a = nil") => "local a",
    assign_to_true_and_nil("local a, b = true, nil") => "local a, b = nil",
    assign_to_nil_and_nil("local a, b = nil, nil") => "local a, b",
);

test_rule_wihout_effects!(
    RemoveNilDeclaration::default(),
    assign_to_true("local a = true"),
    assign_to_nil_and_true("local a = nil, true"),
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_nil_declaration',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_nil_declaration'").unwrap();
}
