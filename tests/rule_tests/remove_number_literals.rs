use darklua_core::rules::{RemoveNumberLiterals, Rule};

test_rule!(
    remove_number_literals,
    RemoveNumberLiterals::default(),
    hexadecimal_integer_literals("local a = 0xABC local b = 0XABC")
        => "local a = 2748 local b = 2748",
    binary_integer_literals("local a = 0b01010101 local b = 0B01010101")
        => "local a = 85 local b = 85",
    decimal_separators("local a = 1_048_576 local b = 0xFFFF_FFFF local c = 0b_0101_0101")
        => "local a = 1048576 local b = 4294967295 local c = 85",
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_number_literals',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_number_literals'").unwrap();
}
