use darklua_core::rules::GroupLocalAssignment;

test_rule!(
    GroupLocalAssignment::default(),
    local_with_no_values("local foo; local bar") => "local foo, bar",
    two_locals("local foo = 1 local bar = 2") => "local foo, bar = 1, 2",
    three_locals("local foo = 1 local bar = 2 local baz = 3") => "local foo, bar, baz = 1, 2, 3",
    local_with_no_value_and_local_with_value("local a local b = 7") => "local a, b = nil, 7",
    local_with_no_values_are_set_to_nil("local a local b = true local c") => "local a, b, c = nil, true, nil"
);

test_rule_wihout_effects!(
    GroupLocalAssignment::default(),
    two_local_using_the_other("local foo = 1 local bar = foo"),
    multiple_return_values("local a, b = call() local c = 0")
);
