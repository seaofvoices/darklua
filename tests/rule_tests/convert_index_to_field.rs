use darklua_core::rules::{ConvertIndexToField, Rule};

test_rule!(
    convert_index_to_field,
    ConvertIndexToField::default(),
    key_is_a_valid_identifier("return var['field']") => "return var.field",
    key_is_a_valid_identifier_with_number("return var['field1']") => "return var.field1",
    key_in_double_index("return var['props'][' ']") => "return var.props[' ']",
    assign_to_a_valid_identifier("var[\"field\"] = call()") => "var.field = call()",
    call_function("object['process'](true)") => "object.process(true)",
    call_method("object['sub']:method(...)") => "object.sub:method(...)",
);

test_rule_without_effects!(
    ConvertIndexToField::default(),
    key_is_empty_string("return var[\"\"]"),
    key_starts_with_space("return var[' bar']"),
    key_ends_with_space("return var['bar ']"),
    key_starts_with_dollar_sign("return var['$$ok']"),
    key_has_dollar_sign("return var['field$end']"),
    key_is_do_keyword("return var['do']"),
    key_starts_with_number("return var['1field']"),
    call_function("object['function'](true)"),
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'convert_index_to_field',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'convert_index_to_field'").unwrap();
}
