use darklua_core::rules::{RemoveUnusedIfBranch, Rule};

test_rule!(
    remove_unused_if_branch,
    RemoveUnusedIfBranch::default(),
    falsy_branch_is_removed("if false then end") => "",
    falsy_branch_with_empty_else_block_is_removed("if false then else end") => "",
    two_falsy_branch_with_empty_else_block_is_removed("if false then elseif false then end") => "",
    falsy_branch_with_else_block_converts_to_do("if false then else return end") => "do return end",
    one_truthy_branch_remove_else_block("if true then break else end") => "do break end",
    remove_falsy_elseif_branch("if foo then break elseif false then end") => "if foo then break end",
    remove_branch_after_truthy_branch("if foo then break elseif true then return elseif foo then end")
        => "if foo then break elseif true then return end"
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_unused_if_branch',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_unused_if_branch'").unwrap();
}
