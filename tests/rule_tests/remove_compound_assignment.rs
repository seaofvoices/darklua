use darklua_core::rules::{RemoveCompoundAssignment, Rule};

test_rule!(
    remove_compound_assignment,
    RemoveCompoundAssignment::default(),
    increase_identifier("a += 1") => "a = a + 1",
    decrease_identifier("a -= 1") => "a = a - 1",
    multiply_identifier("a *= 2") => "a = a * 2",
    divide_identifier("a /= 2") => "a = a / 2",
    mod_identifier("a %= 2") => "a = a % 2",
    exp_identifier("a ^= 2") => "a = a ^ 2",
    concat_identifier("a ..= 'suffix'") => "a = a .. 'suffix'",
    increase_index_without_side_effect("a[prop] += 1") => "a[prop] = a[prop] + 1",
    increase_index_with_true("a[true] += 1") => "a[true] = a[true] + 1",
    increase_index_with_false("a[false] += 1") => "a[false] = a[false] + 1",
    increase_field("a.counter += 1") => "a.counter = a.counter + 1",
    increase_index_with_side_effects_in_index("a[call()] += 1")
        => "do local __DARKLUA_VAR = call() a[__DARKLUA_VAR] = a[__DARKLUA_VAR] + 1 end",
    increase_index_with_side_effects_in_prefix("object[call()][key] += 1")
        => "do local __DARKLUA_VAR = object[call()] __DARKLUA_VAR[key] = __DARKLUA_VAR[key] + 1 end",
    increase_index_with_side_effects_in_prefix_and_index("object[call()][getKey()] += 1")
        => "do local __DARKLUA_VAR, __DARKLUA_VAR0 = object[call()], getKey() __DARKLUA_VAR[__DARKLUA_VAR0] = __DARKLUA_VAR[__DARKLUA_VAR0] + 1 end",
    nested_field_expressions("var.object.prop += 1")
        => "do local __DARKLUA_VAR = var.object __DARKLUA_VAR.prop = __DARKLUA_VAR.prop + 1 end",
    consecutive_nested_field_assignments("a.object.counter += 1 b.object.counter -= 1")
        => "do local __DARKLUA_VAR = a.object __DARKLUA_VAR.counter = __DARKLUA_VAR.counter + 1 end do local __DARKLUA_VAR0 = b.object __DARKLUA_VAR0.counter = __DARKLUA_VAR0.counter - 1 end",
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_compound_assignment',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_compound_assignment'").unwrap();
}
