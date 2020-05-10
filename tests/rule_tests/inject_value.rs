use darklua_core::rules::{InjectGlobalValue, Rule};

test_rule!(
    InjectGlobalValue::nil("foo"),
    inject_nil("return foo") => "return nil"
);

test_rule!(
    InjectGlobalValue::boolean("foo", true),
    inject_true("return foo") => "return true"
);

test_rule!(
    InjectGlobalValue::boolean("foo", false),
    inject_false("return foo") => "return false"
);

test_rule!(
    InjectGlobalValue::string("foo", "bar"),
    inject_string("return foo") => "return 'bar'"
);

test_rule!(
    InjectGlobalValue::float("foo", 10.0),
    inject_integer("return foo") => "return 10"
);

test_rule!(
    InjectGlobalValue::float("foo", -1.0),
    inject_negative_integer("return foo") => "return -1"
);

test_rule!(
    InjectGlobalValue::nil("foo"),
    can_replace_variable_when_out_of_scope_string("do local foo end return foo")
        => "do local foo end return nil"
);

test_rule_wihout_effects!(
    InjectGlobalValue::nil("foo"),
    does_not_override_local_variable("local foo return foo")
);

#[test]
fn deserialize_from_object_notation_without_value() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
    }"#).unwrap();
}

#[test]
fn deserialize_from_object_notation_with_true_value() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: true,
    }"#).unwrap();
}

#[test]
fn deserialize_from_object_notation_with_false_value() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: false,
    }"#).unwrap();
}

#[test]
fn deserialize_from_object_notation_with_string_value() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: 'hello',
    }"#).unwrap();
}

#[test]
fn deserialize_from_object_notation_with_integer_value() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: 1,
    }"#).unwrap();
}

#[test]
fn deserialize_from_object_notation_with_negative_integer_value() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: -1,
    }"#).unwrap();
}

#[test]
fn deserialize_from_object_notation_with_float_value() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: 0.5,
    }"#).unwrap();
}
