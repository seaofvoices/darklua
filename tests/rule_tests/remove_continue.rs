use darklua_core::rules::{RemoveContinue, Rule};

test_rule!(
    remove_continue,
    RemoveContinue::default(),
    continue_inside_numeric_for("for i = 1, 10 do continue end") => "for i = 1, 10 do repeat break until true end",
	continue_and_break_inside_numeric_for("for i = 1, 10 do if i > 5 then break end if i % 2 == 0 then continue end end")
		=> "for i, 10 do local DARKLUA_REMOVE_CONTINUE_break1231231 = false repeat if i > 5 then DARKLUA_REMOVE_CONTINUE_break1231231 = true break end if i % 2 == 0 then break end end"
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_continue',
    }"#,
    )
    .unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'remove_continue'").unwrap();
}
