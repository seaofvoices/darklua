use darklua_core::rules::{ConvertLUXToRoactCode, Rule};

const MERGE: &'static str = "_DARKLUA_SHALLOW_MERGE";
const MERGE_IMPL: &'static str = r#"
local function _DARKLUA_SHALLOW_MERGE(...)
    local result = {}

    for index = 1, select('#', ...) do
        for key, value in pairs(select(index, ...)) do
            result[key] = value
        end
    end

    return result
end
"#;
const MERGE_MIXED: &'static str = "_DARKLUA_MERGE_MIXED";
const MERGE_MIXED_IMPL: &'static str = r#"
local function _DARKLUA_MERGE_MIXED(...)
    local result = {}

    for index = 1, select('#', ...) do
        local children = select(index, ...)
        local length = #children

        for key, value in pairs(children) do
            if typeof(key) == 'number' and key > 0 and key <= length and key % 1 == 0 then
                table.insert(result, value)
            else
                result[key] = value
            end
        end
    end

    return result
end
"#;

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
        => &format!(
            "{} return Roact.createElement('Frame', {}(baseProps, forwardProps))",
            MERGE_IMPL,
            MERGE
        ),
    frame_with_attribute_followed_by_spread("return <Frame foo='bar' {... forwardProps}/>")
        => &format!(
            "{} return Roact.createElement('Frame', {}({{ foo = 'bar' }}, forwardProps))",
            MERGE_IMPL,
            MERGE,
        ),
    frame_with_spread_followed_by_attribute("return <Frame {... baseProps} foo='bar'/>")
        => &format!(
            "{} return Roact.createElement('Frame', {}(baseProps, {{ foo = 'bar' }}))",
            MERGE_IMPL,
            MERGE,
        ),
    fragment_with_single_spread_child_expression("return <>{... list}</>")
        => "return Roact.createFragment(list)",
    fragment_with_two_spread_child_expression("return <>{... listA} {... listB}</>")
        => &format!(
            "{} return Roact.createFragment({}(listA, listB))",
            MERGE_MIXED_IMPL,
            MERGE_MIXED,
        ),
    fragment_with_two_spread_child_expression_separated_by_a_frame("return <>{... listA}<Frame/>{... listB}</>")
        => &format!(
            "{} return Roact.createFragment({}(listA, {{ Roact.createElement('Frame') }}, listB))",
            MERGE_MIXED_IMPL,
            MERGE_MIXED,
        ),
    fragment_with_value_child("return <>{element}</>")
        => "return Roact.createFragment({element})",
    fragment_with_two_value_children("return <>{elementA}{elementB}</>")
        => "return Roact.createFragment({elementA, elementB})",
    fragment_with_two_value_children_separated_by_a_frame("return <>{elementA}<Frame/>{elementB}</>")
        => "return Roact.createFragment({elementA, Roact.createElement('Frame'), elementB})",
    fragment_with_two_value_children_separated_by_a_frame_with_a_key("return <>{elementA}<Frame roact:key='foo'/>{elementB}</>")
        => "return Roact.createFragment({elementA, foo = Roact.createElement('Frame'), elementB})",
    fragment_with_two_value_children_separated_by_spread_expression("return <>{elementA}{...middleChildren}{elementB}</>")
        => &format!(
            "{} return Roact.createFragment({}({{ elementA }}, middleChildren, {{ elementB }}))",
            MERGE_MIXED_IMPL,
            MERGE_MIXED,
        ),
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
