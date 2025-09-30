use darklua_core::rules::{ComputeExpression, Rule};

test_rule!(
    compute_expression,
    ComputeExpression::default(),
    binary_true_and_false("return true and false") => "return false",
    binary_true_and_call("return true and call()") => "return call()",
    binary_false_and_true("return false and true") => "return false",
    binary_false_and_variable("return false and var") => "return false",
    binary_false_and_call("return false and func()") => "return false",
    binary_true_or_call("return true or func()") => "return true",
    binary_true_or_function("return false or function() print('ok') end") => "return function() print('ok') end",
    binary_false_or_call("return false or call()") => "return call()",
    binary_nil_or_call("return nil or call()") => "return call()",
    binary_number_equals("return 1 == 1") => "return true",
    binary_number_equals_in_different_notation("return 1 == 1.0") => "return true",
    binary_number_equals_in_different_exponent_notation("return 2.5e3 == 25e2") => "return true",
    binary_table_or_call("return {} or func()") => "return {}",
    true_and_func_or_call("return true and function() end or call()") => "return function() end",
    nil_and_call_or_func("return nil and call() or function() end") => "return function() end",
    number_addition("return 1 + 2") => "return 3",
    multiple_addition("return 1 + 2 + 5") => "return 8",
    division("return 1/3") => "return 0.3333333333333333",
    division_test("return 3 * 0.3333333333333333") => "return 1",
    multiply_small_number("return 2 * 1e-50") => "return 2E-50",
    unary_minus_number("return -1") => "return -1",
    if_expression_always_true("return if true then 'is true' else 'is false'") => "return 'is true'",
    if_expression_always_true_with_dead_branch_has_side_effects("return if true then 'is true' else call()")
        => "return 'is true'",
    if_expression_always_false("return if false then 'is true' else 'is false'") => "return 'is false'",
    if_expression_always_false_with_dead_branch_has_side_effects("return if false then call() else 'is false'")
        => "return 'is false'",
    if_expression_elseif_always_true("return if false then 'is true' elseif 1 == 1 then 'is equal' else nil")
        => "return 'is equal'",
    if_expression_elseif_always_false("return if false then 'is true' elseif 1 == 2 then 'is equal' else nil")
        => "return nil",
    preserve_negative_zero("return -0") => "return -0",
    addition_preserve_negative_zero("return -0 + -0") => "return -0",
    subtract_preserve_negative_zero("return -0 - 0") => "return -0",
    // identifiers cases
    declared_bool_constant("local var = true return var") => "local var = true return true",
    declared_string_constant("local var = 'help' return var") => "local var = 'help' return 'help'",
    declared_number_constant("local var = 1.1 return var") => "local var = 1.1 return 1.1",
    // math library cases
    math_abs_neg_1("return math.abs(-1)") => "return 1",
    // math.cos
    math_cos_pi("return math.cos(math.pi)") => "return -1",
    math_cos_0("return math.cos(0)") => "return 1",
    math_cos_0_string("return math.cos('0')") => "return 1",
    // math.sin
    math_sin_0("return math.sin(0)") => "return 0",
    math_sin_0_string("return math.sin('0')") => "return 0",
    // math.tan
    math_tan_0("return math.tan(0)") => "return 0",
    math_tan_0_string("return math.tan('0')") => "return 0",
    // math.sign
    math_sign_10("return math.sign(10)") => "return 1",
    math_sign_neg_0_5("return math.sign(-0.5)") => "return -1",
    math_sign_0("return math.sign(0)") => "return 0",
    math_sign_inf("return math.sign(1/0)") => "return 1",
    math_sign_neg_inf("return math.sign(-1/0)") => "return -1",
    math_sign_nan("return math.sign(0/0)") => "return 0",
    math_sign_0_string("return math.sign('0')") => "return 0",
    // math.sqrt
    math_sqrt("return math.sqrt(16)") => "return 4",
    math_sqrt_64_string("return math.sqrt('64')") => "return 8",
    math_sqrt_0("return math.sqrt(0)") => "return 0",
    math_sqrt_inf("return math.sqrt(1/0)") => "return 1/0",
    math_sqrt_neg_inf("return math.sqrt(-1/0)") => "return 0/0",
    math_sqrt_nan("return math.sqrt(0/0)") => "return 0/0",
    // math.pow
    math_pow_2_3("return math.pow(2, 3)") => "return 8",
    math_pow_8_neg_1("return math.pow(8, -1)") => "return 0.125",
    math_pow_2_3_string("return math.pow(2, '3')") => "return 8",
    math_pow_0_0_string("return math.pow(0, '0')") => "return 1",
    math_pow_inf_inf("return math.pow(1/0, 1/0)") => "return 1/0",
    math_pow_neg_inf_neg_inf("return math.pow(-1/0, -1/0)") => "return 0",
    // math.exp
    math_exp_0("return math.exp(0)") => "return 1",
    math_exp_0_string("return math.exp('0')") => "return 1",
    // math.rad
    math_rad_0("return math.rad(0)") => "return 0",
    math_rad_0_string("return math.rad('0')") => "return 0",
    // math.deg
    math_deg_0("return math.deg(0)") => "return 0",
    math_deg_pi("return math.deg(math.pi)") => "return 180",
    math_deg_0_string("return math.deg('0')") => "return 0",
);

test_rule_without_effects!(
    ComputeExpression::default(),
    if_expression_unknown_condition("return if condition then func() else func2()"),
    math_rad_no_arguments("return math.rad()"),
    math_sin_no_arguments("return math.sin()"),
    math_cos_no_arguments("return math.cos()"),
    math_tan_no_arguments("return math.tan()"),
    math_sqrt_no_arguments("return math.sqrt()"),
    math_pow_no_arguments("return math.pow()"),
    math_exp_no_arguments("return math.exp()"),
    math_deg_no_arguments("return math.deg()"),
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'compute_expression',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'compute_expression'").unwrap();
}
