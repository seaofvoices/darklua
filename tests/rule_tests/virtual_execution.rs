use darklua_core::rules::{Rule, VirtualExecution};

test_rule!(
    virtual_execution,
    VirtualExecution::default(),
    binary_true_and_false("return true and false") => "return false",
    binary_false_and_true("return false and true") => "return false",
    binary_false_and_variable("return false and var") => "return false",
    binary_false_and_call("return false and func()") => "return false",
    binary_true_or_call("return true or func()") => "return true",
    binary_true_or_function("return false or function() print('ok') end") => "return function() print('ok') end",
    binary_table_or_call("return {} or func()") => "return {}",
    true_and_func_or_call("return true and function() end or call()") => "return function() end",
    nil_and_call_or_func("return nil and call() or function() end") => "return function() end",
    number_addition("return 1 + 2") => "return 3",
    multiple_addition("return 1 + 2 + 5") => "return 8",
    division("return 1/3") => "return 0.3333333333333333",
    division_test("return 3 * 0.3333333333333333") => "return 1",
    multiply_small_number("return 2 * 1e-50") => "return 2E-50",
    unary_minus_number("return -1") => "return -1",
    unary_not_true("return not true") => "return false",
    unary_on_parenthese("return -(20-10)") => "return -10",
    // cases with identifiers
    local_constant_boolean("local a = true return a") => "local a = true return true",
    local_mutated_boolean("local a = true a = false return a") => "local a = true a = false return false",
    local_constant_in_parenthese("local a = true return (a)") => "local a = true return true",
    table_property_field("local exports = { property = 10 } local a = exports.property return a")
        => "local exports = { property = 10 } local a = 10 return 10",
    table_index_field("local exports = { [''] = false } return exports['']")
        => "local exports = { [''] = false } return false",
    sum_elements_in_list(
        "local list = {2, 4, 6} local sum = 0 for i = 1, 3 do sum = sum + list[i] end return sum"
    ) => "local list = {2, 4, 6} local sum = 0 for i = 1, 3 do sum = sum + list[i] end return 12",
    unary_on_identifier("local a = 30 return -a") => "local a = 30 return -30",
    local_mutated_identifier("local a = true local b = a a = false return b")
        => "local a = true local b = true a = false return true",
    // tables
    mutated_table_by_reference("local a = {} local b = a b.prop = true return a.prop")
        => "local a = {} local b = a b.prop = true return true",
    missing_entry_in_table("local a = {} return a.var") => "local a = {} return nil",
    inline_field_entry("local flag = true local a = { var = flag }") => "local flag = true local a = { var = true }",
    inline_key_entry("local name = 'prop' local a = { [name] = true }") => "local name = 'prop' local a = { ['prop'] = true }",
    // functions
    immediate_function_call("return (function() return true end)()") => "return true",
    call_function_identifier("local function getValue() return 'value' end return getValue()")
        => "local function getValue() return 'value' end return 'value'",
    sum_function("local function sum(a, b) return a + b end return sum(3, 4)")
        => "local function sum(a, b) return a + b end return 7",
    assign_function_in_table("local a = {} a.foo = function() return false end return a.foo()")
        => "local a = {} a.foo = function() return false end return false",
    missing_param_coerce_to_nil("local function getFirst(a) return a end return getFirst()")
        => "local function getFirst(a) return a end return nil",
    function_mutates_state("local var = 1 local function increment(a) var = var + a end increment(2) return var")
        => "local var = 1 local function increment(a) var = var + a end increment(2) return 3",
    // immediate_function_call_returning_multiple_value("return (function() return true, false end)()") => "return true, false",
    immediate_function_call_returning_multiple_value_keeps_first(
        "return ((function() return true, false end)())"
    ) => "return true",

);

test_rule!(
    virtual_execution_with_roblox_bit32,
    json5::from_str::<Box<dyn Rule>>(
        "{ rule: 'virtual_execution', includes: ['roblox-bit32'] }"
    ).unwrap(),
    call_bit32_library_band("return bit32.band(1, 0xff)") => "return 1",
);

test_rule!(
    virtual_execution_with_roblox_math,
    json5::from_str::<Box<dyn Rule>>(
        "{ rule: 'virtual_execution', includes: ['roblox-math'] }"
    ).unwrap(),
    call_math_library_abs("return math.abs(-12)") => "return 12",
    call_math_library_abs_by_reference("local abs = math.abs return abs(-0.5)")
        => "local abs = math.abs return 0.5",
);

test_rule!(
    virtual_execution_with_roblox_string,
    json5::from_str::<Box<dyn Rule>>(
        "{ rule: 'virtual_execution', includes: ['roblox-string'] }"
    ).unwrap(),
    string_len("return string.len('value')") => "return 5",
    string_lower("return string.lower('HEY')") => "return 'hey'",
    string_upper("return string.upper('click')") => "return 'CLICK'",
);

test_rule!(
    virtual_execution_with_tonumber,
    json5::from_str::<Box<dyn Rule>>(
        "{ rule: 'virtual_execution', includes: ['tonumber'] }"
    ).unwrap(),
    on_true("return tonumber(true)") => "return nil",
    on_negative_number("return tonumber(-12)") => "return -12",
    on_string_number("return tonumber('100.5')") => "return 100.5",
);

test_rule!(
    virtual_execution_with_tostring,
    json5::from_str::<Box<dyn Rule>>(
        "{ rule: 'virtual_execution', includes: ['tostring'] }"
    ).unwrap(),
    on_true("return tostring(true)") => "return 'true'",
    on_number("return tostring(13)") => "return '13'",
);

test_rule!(
    virtual_execution_with_type,
    json5::from_str::<Box<dyn Rule>>(
        "{ rule: 'virtual_execution', includes: ['type'] }"
    ).unwrap(),
    on_true("return type(true)") => "return 'boolean'",
    on_negative_number("return type(-12)") => "return 'number'",
);

test_rule_without_effects!(
    VirtualExecution::default(),
    potential_table_mutation("local t = { prop = 7 } callback(t) return t.prop"),
    assign_to_unknown_key_blurs_table_value("local a = {} a[unknown] = true return a.var"),
    table_passed_to_unknown_function("local a = { prop = true } callback(a) return a.prop"),
    function_passed_to_function_that_mutates_state(
        "local var = true local function mutateVar() var = not var end callback(mutateVar) return var"
    ),
    keeps_call_with_side_effect("local function call() trigger() return true end return call()")
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'virtual_execution',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'virtual_execution'").unwrap();
}
