mod utils;

use darklua_core::{process, Options, Resources};

use pretty_assertions::assert_eq;

use utils::memory_resources;

const ANY_CODE: &str = "do end return true";
const ANY_CODE_DEFAULT_PROCESS: &str = "return true";

#[test]
fn apply_default_config_in_place() {
    let resources = memory_resources!(
        "src/test.lua" => ANY_CODE,
    );

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

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

    process(&resources, Options::new("src").with_output("output"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(
        resources.get("output/test.lua").unwrap(),
        ANY_CODE_DEFAULT_PROCESS
    );
}

#[test]
fn apply_default_config_to_output_from_file_in_directory() {
    let resources = memory_resources!(
        "src/test.lua" => ANY_CODE,
        "output/placeholder.txt" => "",
    );

    process(
        &resources,
        Options::new("src/test.lua").with_output("output"),
    )
    .unwrap()
    .result()
    .unwrap();

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

    process(&resources, Options::new("src").with_output("output"))
        .unwrap()
        .result()
        .unwrap();

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
    .unwrap()
    .result()
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
    .unwrap()
    .result()
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

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

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

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return 1");
}

#[test]
fn use_default_json_config_in_place_with_apply_to_files_filter() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        "src/test2.lua" => "return _G.VALUE",
        ".darklua.json" => "{ \"rules\": [ { \"rule\": \"inject_global_value\", \"apply_to_files\": [\"**/test.lua\"], \"identifier\": \"VALUE\", \"value\": 1 } ] }",
    );

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return 1");
    assert_eq!(resources.get("src/test2.lua").unwrap(), "return _G.VALUE");
}

#[test]
fn use_default_json_config_in_place_with_root_level_apply_to_files_filter() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        "src/test2.lua" => "return _G.VALUE",
        ".darklua.json" => "{ \"apply_to_files\": [\"**/test.lua\"], \"rules\": [ { \"rule\": \"inject_global_value\", \"identifier\": \"VALUE\", \"value\": 1 } ] }",
    );

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return 1");
    assert_eq!(resources.get("src/test2.lua").unwrap(), "return _G.VALUE");
}

#[test]
fn use_default_json_config_in_place_with_apply_to_files_filter_all() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        "src/test2.lua" => "return _G.VALUE",
        ".darklua.json" => "{ \"rules\": [ { \"rule\": \"inject_global_value\", \"apply_to_files\": [\"src/**\"], \"identifier\": \"VALUE\", \"value\": 1 } ] }",
    );

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return 1");
    assert_eq!(resources.get("src/test2.lua").unwrap(), "return 1");
}

#[test]
fn use_default_json_config_in_place_with_skip_files_filter() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        "src/test2.lua" => "return _G.VALUE",
        ".darklua.json" => "{ \"rules\": [ { \"rule\": \"inject_global_value\", \"skip_files\": [\"**/test.lua\"], \"identifier\": \"VALUE\", \"value\": 1 } ] }",
    );

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return _G.VALUE");
    assert_eq!(resources.get("src/test2.lua").unwrap(), "return 1");
}

#[test]
fn use_default_json_config_in_place_with_root_level_skip_files_filter() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        "src/test2.lua" => "return _G.VALUE",
        ".darklua.json" => "{ \"skip_files\": [\"**/test.lua\"], \"rules\": [ { \"rule\": \"inject_global_value\", \"identifier\": \"VALUE\", \"value\": 1 } ] }",
    );

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return _G.VALUE");
    assert_eq!(resources.get("src/test2.lua").unwrap(), "return 1");
}

#[test]
fn use_default_json_config_in_place_with_apply_to_files_and_skip_files_filter() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        "src/test2.lua" => "return _G.VALUE",
        ".darklua.json" => "{ \"rules\": [ { \"rule\": \"inject_global_value\", \"apply_to_files\": [\"src/**\"], \"skip_files\": [\"**/test.lua\"], \"identifier\": \"VALUE\", \"value\": 1 } ] }",
    );

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return _G.VALUE");
    assert_eq!(resources.get("src/test2.lua").unwrap(), "return 1");
}

#[test]
fn use_default_json5_config_in_place() {
    let resources = memory_resources!(
        "src/test.lua" => "return _G.VALUE",
        ".darklua.json5" => "{ rules: [ { rule: 'inject_global_value', identifier: 'VALUE', value: 'Hello' } ] }",
    );

    process(&resources, Options::new("src"))
        .unwrap()
        .result()
        .unwrap();

    assert_eq!(resources.get("src/test.lua").unwrap(), "return 'Hello'");
}

mod errors {
    use std::path::{Path, PathBuf};

    use darklua_core::{
        nodes::Block,
        rules::{
            Context, Rule, RuleConfiguration, RuleConfigurationError, RuleMetadata,
            RuleProcessResult, RuleProperties,
        },
        Configuration, WorkerTree,
    };

    use super::*;

    fn assert_errors(snapshot_name: &'static str, resources: &Resources, options: Options) {
        let errors = process(resources, options)
            .map_err(|err| vec![err])
            .and_then(WorkerTree::result)
            .unwrap_err();

        let errors_display = errors
            .into_iter()
            .map(|err| format!("- {}", err).replace('\\', "/"))
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(snapshot_name, errors_display);
    }

    #[test]
    fn snapshot_simple_cyclic_work_error() {
        let resources = memory_resources!(
            "src/a.lua" => "return 'module a'",
            "src/b.lua" => "return 'module b'",
        );

        #[derive(Debug, Default)]
        struct CustomRule {
            metadata: RuleMetadata,
        }

        impl RuleConfiguration for CustomRule {
            fn configure(
                &mut self,
                _properties: RuleProperties,
            ) -> Result<(), RuleConfigurationError> {
                Ok(())
            }

            fn get_name(&self) -> &'static str {
                "custom-rule"
            }

            fn serialize_to_properties(&self) -> RuleProperties {
                Default::default()
            }

            fn set_metadata(&mut self, metadata: RuleMetadata) {
                self.metadata = metadata;
            }

            fn metadata(&self) -> &RuleMetadata {
                &self.metadata
            }
        }

        impl Rule for CustomRule {
            fn process(&self, _: &mut Block, _: &Context) -> RuleProcessResult {
                Ok(())
            }

            fn require_content(&self, _: &Path, _: &Block) -> Vec<PathBuf> {
                vec!["src/a.lua".into(), "src/b.lua".into()]
            }
        }

        let rule: Box<dyn Rule> = Box::new(CustomRule::default());

        assert_errors(
            "simple_cyclic_work_error",
            &resources,
            Options::new("src").with_configuration(Configuration::empty().with_rule(rule)),
        );
    }

    #[test]
    fn snapshot_missing_configuration_file() {
        let resources = memory_resources!(
            "src/init.lua" => "return ''",
        );

        assert_errors(
            "missing_configuration_file",
            &resources,
            Options::new("src").with_configuration_at("missing/config.json"),
        );
    }

    #[test]
    fn snapshot_multiple_configuration_file_found() {
        let resources = memory_resources!(
            "src/init.lua" => "return ''",
            ".darklua.json" => "{ rules: [] }",
            ".darklua.json5" => "{ rules: [] }",
        );

        assert_errors(
            "multiple_configuration_file_found",
            &resources,
            Options::new("src"),
        );
    }
}
