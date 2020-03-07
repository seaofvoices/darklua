use darklua_core::rules::RemoveMethodDefinition;

test_rule!(
    RemoveMethodDefinition::default(),
    name_without_method("function foo() end") => "function foo() end",
    name_with_method("function foo:bar() end") => "function foo.bar(self) end",
    name_with_field_and_method("function foo.bar:baz() end") => "function foo.bar.baz(self) end",
    with_arguments("function foo:bar(a, b, c) end") => "function foo.bar(self, a, b, c) end",
    variadic_function("function foo:bar(...) end") => "function foo.bar(self, ...) end",
    variadic_with_arguments("function foo:bar(a, b, c, ...) end") => "function foo.bar(self, a, b, c, ...) end"
);
