use darklua_core::rules::{ComputeExpression, Rule};

test_rule!(
    ComputeExpression::default(),
    binary_true_and_false("return true and false") => "return false",
    number_addition("return 1 + 2") => "return 3",
    multiple_addition("return 1 + 2 + 5") => "return 8",
    division("return 1/3") => "return 0.3333333333333333",
    division_test("return 3 * 0.3333333333333333") => "return 1",
    multiply_small_number("return 2 * 1e-50") => "return 2E-50"
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'compute_expression',
    }"#).unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'compute_expression'").unwrap();
}
