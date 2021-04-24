use darklua_core::rules::{ConvertLUXToRoactCode, Rule};

test_rule!(
    ConvertLUXToRoactCode::default(),
    empty_fragment("return <></>") => "return Roact.createFragment({})",
    frame_host_element("return <Frame/>") => "return Roact.createElement(\"Frame\")",
    custom_frame_element("local Frame = ... return <Frame/>")
        => "local Frame = ... return Roact.createElement(Frame)",
    frame_boolean_flag_attribute("return <Frame Visible/>")
        => "return Roact.createElement(\"Frame\", { Visible = true })",
    text_label_single_quote_attribute("return <TextLabel Text='hello!'/>")
        => "return Roact.createElement(\"TextLabel\", { Text = 'hello!' })",
    text_label_double_quote_attribute("return <TextLabel Text=\"hello!\"/>")
        => "return Roact.createElement(\"TextLabel\", { Text = 'hello!' })",
    frame_boolean_attribute("return <Frame Visible={false}/>")
        => "return Roact.createElement(\"Frame\", { Visible = false })",
    frame_activated_event_attribute("return <Frame event:Activated={true}/>")
        => "return Roact.createElement(\"Frame\", { [Roact.Event.Activated] = true })",
    frame_changed_event_attribute("return <Frame changed:AbsoluteSize={true}/>")
        => "return Roact.createElement(\"Frame\", { [Roact.Changed.AbsoluteSize] = true })",
    frame_in_fragment("return <><Frame/></>")
        => "return Roact.createFragment({ Roact.createElement('Frame') })",
    frame_with_key_in_fragment("return <><Frame roact:key='foo'/></>")
        => "return Roact.createFragment({ foo = Roact.createElement('Frame') })",
    frame_with_ref("return <Frame roact:ref={refValue}/>")
        => "return Roact.createElement('Frame', { [Roact.Ref] = refValue })",
    frame_with_only_one_spread_attribute("return <Frame {... forwardProps}/>")
        => "return Roact.createElement('Frame', forwardProps)",
    frame_with_two_spread_attribute("return <Frame {... baseProps} {... forwardProps}/>")
        => "return Roact.createElement('Frame', forwardProps)",
);

test_rule_wihout_effects!(
    ConvertLUXToRoactCode::default(),
    does_not_mutate_true_expression("return true")
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(r#"{
        rule: 'convert_lux_to_roact_code',
    }"#).unwrap();
}

#[test]
fn deserialize_from_string() {
    json5::from_str::<Box<dyn Rule>>("'convert_lux_to_roact_code'").unwrap();
}
