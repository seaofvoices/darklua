use darklua_core::rules::Rule;

test_rule!(
    remove_generalized_iteration,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
            rule: 'remove_generalized_iteration',
			runtime_variable_format: '{name}'
        }"#
    ).unwrap(),
    generic_for("for i,v in {1,2,3} do end")
        => "do local iter,invar,control={1,2,3} if type(iter)=='table' then local _m=getmetatable(iter) if type(_m)=='table' and type(_m.__iter)=='function' then iter,invar,control=_m.__iter() else iter,invar,control=pairs(iter) end end for i,v in iter,invar,control do end end"
);

#[test]
fn deserialize_from_object_notation() {
    json5::from_str::<Box<dyn Rule>>(
        r#"{
        rule: 'remove_generalized_iteration',
		runtime_variable_format: '{name}'
    }"#,
    )
    .unwrap();
}
