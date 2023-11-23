use darklua_core::rules::{InjectGlobalValue, Rule};

test_rule!(
    inject_global_nil,
    InjectGlobalValue::nil("foo"),
    inject_nil("return foo") => "return nil",
    inject_nil_from_global_table("call(_G.foo)") => "call(nil)",
    inject_nil_from_global_table_index("call(_G['foo'])") => "call(nil)",
    can_replace_variable_when_out_of_scope_string("do local foo end return foo")
        => "do local foo end return nil",
    inject_nil_from_global_table_even_if_redefined("local foo return _G.foo") => "local foo return nil",
);

test_rule!(
    inject_global_true,
    InjectGlobalValue::boolean("foo", true),
    inject_true("return foo") => "return true",
    inject_true_from_global_table("local a = _G.foo") => "local a = true",
);

test_rule!(
    inject_global_false,
    InjectGlobalValue::boolean("foo", false),
    inject_false("return foo") => "return false",
    inject_false_from_global_table("if _G.foo then return end") => "if false then return end",
);

test_rule!(
    inject_global_string,
    InjectGlobalValue::string("foo", "bar"),
    inject_string("return foo") => "return 'bar'",
    inject_string_from_global_table("var = _G.foo") => "var = 'bar'",
);

test_rule!(
    inject_global_number,
    InjectGlobalValue::number("foo", 10.0),
    inject_integer("return foo") => "return 10",
    inject_integer_from_global_table("return _G.foo") => "return 10",
);

test_rule!(
    inject_global_negative_number,
    InjectGlobalValue::number("foo", -1.0),
    inject_negative_integer("return foo") => "return -1",
    inject_negative_integer_from_global_table("return _G.foo + 1") => "return -1 + 1",
);

test_rule_without_effects!(
    InjectGlobalValue::nil("foo"),
    does_not_override_local_variable("local foo return foo"),
    does_not_inline_if_global_table_is_redefined("local _G return _G.foo"),
);

#[test]
fn deserialize_from_object_notation_without_value() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_object_notation_with_true_value() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: true,
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_object_notation_with_false_value() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: false,
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_object_notation_with_string_value() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: 'hello',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_object_notation_with_integer_value() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: 1,
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_object_notation_with_negative_integer_value() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: -1,
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_object_notation_with_float_value() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'foo',
        value: 0.5,
    }"#,
    )
    .unwrap();
}

test_rule!(
    inject_global_large_integer_e19,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'num',
        value: 1E19,
    }"#,
    ).unwrap(),
    inject_negative_integer("return _G.num") => "return 1E19",
);

test_rule!(
    inject_global_large_integer_e20,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'num',
        value: 1e20,
    }"#,
    ).unwrap(),
    inject_negative_integer("return _G.num") => "return 1E20",
);

test_rule!(
    inject_global_large_integer_e42,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'num',
        value: 1e42,
    }"#,
    ).unwrap(),
    inject_negative_integer("return _G.num") => "return 1E42",
);

test_rule!(
    inject_global_large_integer_e49,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'inject_global_value',
        identifier: 'num',
        value: 1e49,
    }"#,
    ).unwrap(),
    inject_negative_integer("return _G.num") => "return 1E49",
);

#[test]
fn deserialize_number_value_too_large() {
    let err = json5::from_str::<Box<dyn Rule>>(
        r#"{
            rule: 'inject_global_value',
            identifier: 'num',
            value: 1e350,
    }"#,
    )
    .unwrap_err();

    pretty_assertions::assert_eq!("error parsing number: too large", err.to_string())
}
