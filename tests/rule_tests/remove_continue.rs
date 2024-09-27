use darklua_core::rules::Rule;

test_rule!(
    remove_continue_without_hash,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
            rule: 'remove_continue',
            no_hash: true
        }"#
    ).unwrap(),
    continue_inside_numeric_for("for i = 1,10 do continue end") => "for i = 1,10 do repeat break until true end",
    continue_inside_generic_for("for i,v in {a,b,c} do continue end") => "for i,v in {a,b,c} do repeat break until true end",
    continue_inside_repeat("repeat continue until true") => "repeat repeat break until true until true",
    continue_inside_while("while true do continue end") => "while true do repeat break until true end",
    continue_break_inside_numeric_for("for i = 1,10 do if true then break end continue end")
        => "for i = 1,10 do local __DARKLUA_REMOVE_CONTINUE_break = false repeat if true then __DARKLUA_REMOVE_CONTINUE_break = true break end break until true if __DARKLUA_REMOVE_CONTINUE_break then break end end",
    continue_break_inside_generic_for("for i,v in {a,b,c} do if true then break end continue end")
        => "for i,v in {a,b,c} do local __DARKLUA_REMOVE_CONTINUE_break = false repeat if true then __DARKLUA_REMOVE_CONTINUE_break = true break end break until true if __DARKLUA_REMOVE_CONTINUE_break then break end end",
    continue_break_inside_repeat("repeat if true then break end continue until true")
        => "repeat local __DARKLUA_REMOVE_CONTINUE_break = false repeat if true then __DARKLUA_REMOVE_CONTINUE_break = true break end break until true if __DARKLUA_REMOVE_CONTINUE_break then break end until true",
    continue_break_inside_while("while true do if true then break end continue end")
        => "while true do local __DARKLUA_REMOVE_CONTINUE_break = false repeat if true then __DARKLUA_REMOVE_CONTINUE_break = true break end break until true if __DARKLUA_REMOVE_CONTINUE_break then break end end",
    break_continue_inside_numeric_for("for i = 1,10 do if true then break elseif false then break end continue end")
        => "for i = 1,10 do local __DARKLUA_REMOVE_CONTINUE_continue = false repeat if true then break elseif false then break end __DARKLUA_REMOVE_CONTINUE_continue = true break until true if not __DARKLUA_REMOVE_CONTINUE_continue then break end end",
    break_continue_inside_generic_for("for i,v in {a,b,c} do if true then break elseif false then break end continue end")
        => "for i,v in {a,b,c} do local __DARKLUA_REMOVE_CONTINUE_continue = false repeat if true then break elseif false then break end __DARKLUA_REMOVE_CONTINUE_continue = true break until true if not __DARKLUA_REMOVE_CONTINUE_continue then break end end",
    break_continue_inside_repeat("repeat if true then break elseif false then break end continue until true")
        => "repeat local __DARKLUA_REMOVE_CONTINUE_continue = false repeat if true then break elseif false then break end __DARKLUA_REMOVE_CONTINUE_continue = true break until true if not __DARKLUA_REMOVE_CONTINUE_continue then break end until true",
    break_continue_inside_while("while true do if true then break elseif false then break end continue end")
        => "while true do local __DARKLUA_REMOVE_CONTINUE_continue = false repeat if true then break elseif false then break end __DARKLUA_REMOVE_CONTINUE_continue = true break until true if not __DARKLUA_REMOVE_CONTINUE_continue then break end end"
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_continue',
		no_hash: false
    }"#,
    )
    .unwrap();
}
