use darklua_core::rules::Rule;

test_rule_with_tokens!(
    append_text_comment_start,
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'append_text_comment',
        text: 'hello',
    }"#).unwrap(),
    empty_do("do end") => "--hello\ndo end",
    local_assign("local a") => "--hello\nlocal a",
    local_assign_with_value("local var = true") => "--hello\nlocal var = true",
    assign_variable("var = true") => "--hello\nvar = true",
    function_call("fn()") => "--hello\nfn()",
    function_call_field("module.fn()") => "--hello\nmodule.fn()",
    function_call_method("object:fn()") => "--hello\nobject:fn()",
    compound_assign("var += 1") => "--hello\nvar += 1",
    function_statement("function fn() end") => "--hello\nfunction fn() end",
    generic_statement("for k, v in {} do end") => "--hello\nfor k, v in {} do end",
    if_statement("if condition then end") => "--hello\nif condition then end",
    local_function("local function fn() end") => "--hello\nlocal function fn() end",
    numeric_for_statement("for i = 1, 10 do end") => "--hello\nfor i = 1, 10 do end",
    repeat_statement("repeat until condition") => "--hello\nrepeat until condition",
    while_statement("while condition do end") => "--hello\nwhile condition do end",
    type_declaration("type Name = string") => "--hello\ntype Name = string",
    exported_type_declaration("export type Name = string") => "--hello\nexport type Name = string",
    break_statement("break") => "--hello\nbreak",
    continue_statement("continue") => "--hello\ncontinue",
    empty_return_statement("return") => "--hello\nreturn",
    return_one_value_statement("return 1") => "--hello\nreturn 1",
);

test_rule_with_tokens!(
    append_text_comment_start_native,
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'append_text_comment',
        text: '!native',
    }"#).unwrap(),
    append_native_direction("return {}") => "--!native\nreturn {}",
);

test_rule_with_tokens!(
    append_text_comment_multiline,
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'append_text_comment',
        text: '1\n2',
    }"#).unwrap(),
    empty_do("do end") => "--[[\n1\n2\n]]\ndo end",
);

test_rule_without_effects!(
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'append_text_comment',
        text: '',
    }"#
    )
    .unwrap(),
    before_local_function("local function foo() foo() end"),
    before_empty_ast(""),
);

test_rule_without_effects!(
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'append_text_comment',
        text: '',
        location: 'end',
    }"#
    )
    .unwrap(),
    after_local_function("local function foo() end"),
    after_empty_ast(""),
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'append_text_comment',
        text: 'content',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string_fails() {
    let err = json5::from_str::<Box<dyn Rule>>(r#"'append_text_comment'"#).unwrap_err();

    pretty_assertions::assert_eq!("missing one field from `text` and `file`", err.to_string())
}
