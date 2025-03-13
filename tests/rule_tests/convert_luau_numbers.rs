use darklua_core::rules::{ConvertLuauNumbers, Rule};

test_rule!(
    convert_luau_numbers,
    ConvertLuauNumbers::default(),
    binary_integer_literals("local a = 0b01010101 local b = 0B01010101 local c = 0b_0101_0101 local d = 0B_0101_0101 local e = 0b__________0101_0101")
        => "local a = 85 local b = 85 local c = 85 local d = 85 local e = 85",
    decimal_separators("local a = 1_048_576 local a1 = 1___048__576__ local b = 0xFFFF_FFFF local c = 0b_0101_0101 local d = 0B_0101_0101")
        => "local a = 1048576 local a1 = 1048576 local b = 0xFFFFFFFF local c = 85 local d = 85",
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'convert_luau_numbers',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'convert_luau_numbers'").unwrap();
}
