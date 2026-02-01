use darklua_core::rules::{RemoveAttribute, Rule};


test_rule_with_tokens!(
    remove_attribute,
    RemoveAttribute::default(),
    remove_simple_attribute("@deprecated function foo() end") => "function foo() end",
    remove_from_local("@test local function baz() end") => "local function baz() end",
    remove_from_expression("local f = @deprecated function() end") => "local f = function() end",
    remove_multiple("@deprecated @native function foo() end") => "function foo() end",
    remove_nested("@outer function foo() @inner local function bar() end end") => "function foo() local function bar() end end"

    // tests for grouped attributes (@[attr1, attr2]) are
    // not included right now because the full-moon parser
);

test_rule_with_tokens!(
    remove_attribute_with_match,
    RemoveAttribute::default().with_match("deprecated").with_match("test"),
    selective_removal("@deprecated function foo() end @native function bar() end @test local function baz() end")
        => "function foo() end @native function bar() end local function baz() end"
);

test_rule_with_tokens!(
    remove_attribute_with_regex,
    RemoveAttribute::default().with_match("test.*"),
    regex_pattern("@test\nfunction foo()\nend\n\n@testonly\nfunction bar()\nend\n\n@native\nfunction baz()\nend") => "\nfunction foo()\nend\n\n\nfunction bar()\nend\n\n@native\nfunction baz()\nend"
);

test_rule_without_effects!(
    RemoveAttribute::default(),
    no_change_when_no_attributes(
        "function foo()\nend\n\nlocal function bar()\nend\n\nlocal baz = function()\nend"
    )
);

test_rule_without_effects!(
    RemoveAttribute::default().with_match("deprecated"),
    no_change_when_match_does_not_match(
        "@native\nfunction foo()\nend\n\n@test\nlocal function bar()\nend"
    )
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
            rule: 'remove_attribute',
        }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>(r#"'remove_attribute'"#).unwrap();
}
