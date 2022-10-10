use darklua_core::{process, Options, Resources};

use pretty_assertions::assert_eq;

macro_rules! memory_resources {
    ($($path:literal => $content:expr),+$(,)?) => ({
        let resources = Resources::from_memory();
        $(
            resources.write($path, $content).unwrap();
        )*
        resources
    });
}

const ANY_CODE: &str = "do end return true";
const ANY_CODE_DEFAULT_PROCESS: &str = "return true";

#[test]
fn apply_default_config_in_place() {
    let resources = memory_resources!(
        "src/test.lua" => ANY_CODE,
    );

    process(&resources, Options::new("src")).unwrap();

    assert_eq!(
        resources.get("src/test.lua").unwrap(),
        ANY_CODE_DEFAULT_PROCESS
    );
}

#[test]
fn apply_default_config_to_output() {
    let resources = memory_resources!(
        "src/test.lua" => ANY_CODE,
    );

    process(&resources, Options::new("src").with_output("output")).unwrap();

    assert_eq!(
        resources.get("output/test.lua").unwrap(),
        ANY_CODE_DEFAULT_PROCESS
    );
}

#[test]
fn apply_default_config_to_output_with_nested_content() {
    let init_lua = "return{}";
    let resources = memory_resources!(
        "src/test.lua" => ANY_CODE,
        "src/impl/init.lua" => init_lua,
    );

    process(&resources, Options::new("src").with_output("output")).unwrap();

    assert_eq!(
        resources.get("output/test.lua").unwrap(),
        ANY_CODE_DEFAULT_PROCESS
    );
    assert_eq!(resources.get("output/impl/init.lua").unwrap(), init_lua);
}

#[test]
fn apply_default_config_to_specific_file() {
    let resources = memory_resources!(
        "src/test.lua" => ANY_CODE,
    );

    process(
        &resources,
        Options::new("src/test.lua").with_output("output/test.lua"),
    )
    .unwrap();

    assert_eq!(
        resources.get("output/test.lua").unwrap(),
        ANY_CODE_DEFAULT_PROCESS
    );
}

#[test]
fn apply_default_config_to_specific_file_and_output_to_directory() {
    let resources = memory_resources!(
        "src/test.lua" => ANY_CODE,
    );

    process(
        &resources,
        Options::new("src/test.lua").with_output("output"),
    )
    .unwrap();

    assert_eq!(
        resources.get("output/test.lua").unwrap(),
        ANY_CODE_DEFAULT_PROCESS
    );
}

#[test]
fn use_provided_config_in_place() {
    let resources = memory_resources!(
        "src/test.lua" => ANY_CODE,
        "config.json" => "",
    );

    process(&resources, Options::new("src")).unwrap();

    assert_eq!(
        resources.get("src/test.lua").unwrap(),
        ANY_CODE_DEFAULT_PROCESS
    );
}

#[test]
fn use_default_json_config_in_place() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        ".darklua.json" => "{ \"rules\": [ { \"rule\": \"inject_global_value\", \"identifier\": \"VALUE\", \"value\": 1 } ] }",
    );

    process(&resources, Options::new("src")).unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return 1");
}

#[test]
fn use_default_json5_config_in_place() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        ".darklua.json5" => "{ rules: [ { rule: 'inject_global_value', identifier: 'VALUE', value: 'Hello' } ] }",
    );

    process(&resources, Options::new("src")).unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return 'Hello'");
}
