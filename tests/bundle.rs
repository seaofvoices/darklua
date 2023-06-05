use darklua_core::{process, Options, Resources};

mod utils;

macro_rules! memory_resources {
    ($($path:literal => $content:expr),+$(,)?) => ({
        let resources = Resources::from_memory();
        $(
            resources.write($path, $content).unwrap();
        )*
        resources
    });
}

const DARKLUA_BUNDLE_ONLY_CONFIG: &str =
    "{ \"rules\": [], \"generator\": \"readable\", \"bundle\": { \"require-mode\": \"path\" } }";

mod without_rules {
    use super::*;

    fn process_main(resources: &Resources, snapshot_name: &'static str) {
        process(
            resources,
            Options::new("src/main.lua").with_output("out.lua"),
        )
        .result()
        .unwrap();

        let main = resources.get("out.lua").unwrap();

        insta::assert_snapshot!(format!("bundle_without_rules_{}", snapshot_name), main);
    }

    fn process_main_with_errors(resources: &Resources, snapshot_name: &str) {
        let errors = process(
            resources,
            Options::new("src/main.lua").with_output("out.lua"),
        )
        .result()
        .unwrap_err();

        let error_display: Vec<_> = errors.into_iter().map(|err| err.to_string()).collect();

        let mut settings = insta::Settings::clone_current();
        settings.add_filter("\\\\", "/");
        settings.bind(|| {
            insta::assert_snapshot!(snapshot_name, error_display.join("\n"));
        });
    }

    mod module_locations {
        use super::*;

        fn process_main_require_value(resources: Resources) {
            // we can re-use the same snapshot because the output file should
            // resolve to the same code
            process_main(&resources, "require_lua_file");
        }

        #[test]
        fn require_lua_file() {
            process_main_require_value(memory_resources!(
                "src/value.lua" => "return true",
                "src/main.lua" => "local value = require('./value.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_lua_file_in_sibling_nested_file() {
            process_main_require_value(memory_resources!(
                "src/constants/value.lua" => "return true",
                "src/main.lua" => "local value = require('./constants/value.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_lua_file_in_parent_directory() {
            process_main_require_value(memory_resources!(
                "value.lua" => "return true",
                "src/main.lua" => "local value = require('../value.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }
        #[test]
        fn require_lua_file_without_extension() {
            process_main_require_value(memory_resources!(
                "src/value.lua" => "return true",
                "src/main.lua" => "local value = require('./value')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_lua_file_in_parent_directory_without_extension() {
            process_main_require_value(memory_resources!(
                "value.lua" => "return true",
                "src/main.lua" => "local value = require('../value')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_luau_file_in_parent_directory_without_extension() {
            process_main_require_value(memory_resources!(
                "value.luau" => "return true",
                "src/main.lua" => "local value = require('../value')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_luau_file_without_extension() {
            process_main_require_value(memory_resources!(
                "src/value.luau" => "return true",
                "src/main.lua" => "local value = require('./value')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_directory_with_init_lua_file() {
            process_main_require_value(memory_resources!(
                "src/value/init.lua" => "return true",
                "src/main.lua" => "local value = require('./value')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_directory_with_init_luau_file() {
            process_main_require_value(memory_resources!(
                "src/value/init.luau" => "return true",
                "src/main.lua" => "local value = require('./value')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_in_parent_directory() {
            process_main_require_value(memory_resources!(
                "value.lua" => "return true",
                "src/main.lua" => "local value = require('../value.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            ));
        }

        #[test]
        fn require_in_packages_directory() {
            process_main_require_value(memory_resources!(
                "packages/value.lua" => "return true",
                "src/main.lua" => "local value = require('Packages/value.lua')",
                ".darklua.json" => "{ \"rules\": [], \"generator\": \"readable\", \"bundle\": { \"require-mode\": { \"name\": \"path\", \"sources\": { \"Packages\": \"./packages\" } } } }",
            ));
        }

        #[test]
        fn require_directory_with_custom_init_file() {
            process_main_require_value(memory_resources!(
                "src/value/__init__.lua" => "return true",
                "src/main.lua" => "local value = require('./value')",
                ".darklua.json" => "{ \"rules\": [], \"generator\": \"readable\", \"bundle\": { \"require-mode\": { \"name\": \"path\", \"module-folder-name\": \"__init__.lua\" } } }",
            ));
        }
    }

    #[test]
    fn require_lua_file_after_declaration() {
        let resources = memory_resources!(
            "src/value.lua" => "return true",
            "src/main.lua" => "local const = 1\nlocal value = require('./value.lua')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_lua_file_after_declaration");
    }

    #[test]
    fn require_lua_file_nested() {
        let resources = memory_resources!(
            "src/constant.lua" => "return 2",
            "src/value.lua" => "local constant = require('./constant.lua')\nreturn constant + constant",
            "src/main.lua" => "local value = require('./value.lua')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_lua_file_nested");
    }

    #[test]
    fn require_lua_file_twice() {
        let resources = memory_resources!(
            "src/constant.lua" => "print('load constant module') return 2",
            "src/value_a.lua" => "print('load value a')\nlocal constant_a = require('./constant.lua')\nreturn constant_a",
            "src/value_b.lua" => "print('load value b')\nlocal constant_b = require('./constant.lua')\nreturn constant_b",
            "src/main.lua" => concat!(
                "local value_a = require('./value_a.lua')\n",
                "local value_b = require('./value_b.lua')\n",
                "print(value_a + value_b)"
            ),
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_lua_file_twice");
    }

    #[test]
    fn require_lua_file_twice_with_different_paths() {
        let resources = memory_resources!(
            "src/constant.lua" => "print('load constant module') return 2",
            "src/a/value_a.lua" => "print('load value a')\nlocal constant_a = require('../constant.lua')\nreturn constant_a",
            "src/value_b.lua" => "print('load value b')\nlocal constant_b = require('./constant.lua')\nreturn constant_b",
            "src/main.lua" => concat!(
                "local value_a = require('./a/value_a.lua')\n",
                "local value_b = require('./value_b.lua')\n",
                "print(value_a + value_b)"
            ),
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_lua_file_twice_with_different_paths");
    }

    #[test]
    fn require_lua_file_with_field_expression() {
        let resources = memory_resources!(
            "src/value.lua" => "return { value = 'oof' }",
            "src/main.lua" => "local value = require('./value.lua').value",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_lua_file_with_field_expression");
    }

    #[test]
    fn require_lua_file_with_statement() {
        let resources = memory_resources!(
            "src/run.lua" => "print('run')\nreturn nil",
            "src/main.lua" => "require('./run.lua')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_lua_file_with_statement");
    }

    #[test]
    fn require_json_file_with_object() {
        let resources = memory_resources!(
            "src/value.json" => "{ \"value\": true }",
            "src/main.lua" => "local value = require('./value.json')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_json_file_with_object");
    }

    #[test]
    fn require_json5_file_with_object() {
        let resources = memory_resources!(
            "src/value.json5" => "{ value: true }",
            "src/main.lua" => "local value = require('./value.json5')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_json_file_with_object");
    }

    #[test]
    fn require_json5_file_as_json_with_object() {
        let resources = memory_resources!(
            "src/value.json" => "{ value: true }",
            "src/main.lua" => "local value = require('./value.json')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_json_file_with_object");
    }

    #[test]
    fn require_toml_with_object() {
        let resources = memory_resources!(
            "src/value.toml" => "name = 'darklua'\nvalue = 10",
            "src/main.lua" => "local value = require('./value.toml')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_toml_with_object");
    }

    #[test]
    fn require_yaml_with_array() {
        let resources = memory_resources!(
            "src/value.yaml" => r#"
- 0
- 100
            "#,
            "src/main.lua" => "local value = require('./value.yaml')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_yaml_with_array");
    }

    #[test]
    fn require_yml_with_object() {
        let resources = memory_resources!(
            "src/value.yml" => r#"
name: darklua
data:
    bool: true
    numbers:
    - 0
    - 100
            "#,
            "src/main.lua" => "local value = require('./value.yml')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "require_yml_with_object");
    }

    #[test]
    fn require_value_and_override_require_function() {
        let resources = memory_resources!(
            "src/value.lua" => "return 1",
            "src/main.lua" => "local value = require('./value') local require = function()end local v = require('v')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main(&resources, "override_require");
    }

    #[test]
    fn require_unknown_module() {
        let resources = memory_resources!(
            "src/main.lua" => "local library = require('@lune/library')",
            ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
        );

        process_main_with_errors(&resources, "require_unknown_module");
    }

    #[test]
    fn require_skip_unknown_module() {
        let resources = memory_resources!(
            "src/main.lua" => "local library = require('@lune/library')",
            ".darklua.json" => "{ \"rules\": [], \"bundle\": { \"require-mode\": \"path\", \"excludes\": [\"@lune/**\"] } }",
        );

        process_main(&resources, "require_skip_unknown_module");
    }

    mod cyclic_requires {
        use super::*;

        fn process_main_with_error(resources: &Resources, snapshot_name: &str) {
            process_main_with_errors(resources, &format!("cyclic_requires__{}", snapshot_name));
        }

        #[test]
        fn simple_direct_cycle() {
            let resources = memory_resources!(
                "src/value1.lua" => "return require('./value2')",
                "src/value2.lua" => "return require('./value1')",
                "src/main.lua" => "local value = require('./value1.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            );

            process_main_with_error(&resources, "simple_direct_cycle");
        }

        #[test]
        fn simple_direct_cycle_in_required_file() {
            let resources = memory_resources!(
                "src/value1.lua" => "return require('./value2')",
                "src/value2.lua" => "return require('./value1')",
                "src/constant.lua" => "return require('./value1.lua')",
                "src/main.lua" => "local value = require('./constant.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            );

            process_main_with_error(&resources, "simple_direct_cycle_in_required_file");
        }

        #[test]
        fn simple_transitive_cycle() {
            let resources = memory_resources!(
                "src/value1.lua" => "return require('./constant')",
                "src/value2.lua" => "return require('./value1')",
                "src/constant.lua" => "return require('./value2.lua')",
                "src/main.lua" => "local value = require('./value1.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            );

            process_main_with_error(&resources, "simple_transitive_cycle");
        }

        #[test]
        fn direct_cycle_in_required_file_with_ok_require() {
            let resources = memory_resources!(
                "src/value1.lua" => "return require('./value2')",
                "src/value2.lua" => "return require('./value1')",
                "src/constant.lua" => "return 1",
                "src/main.lua" => "local constant = require('./constant.lua')\nlocal value = require('./value1.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            );

            process_main_with_error(&resources, "direct_cycle_in_required_file_with_ok_require");
        }

        #[test]
        fn two_different_direct_cycles() {
            let resources = memory_resources!(
                "src/value1.lua" => "return require('./value2')",
                "src/value2.lua" => "return require('./value1')",
                "src/constant1.lua" => "return require('./constant2')",
                "src/constant2.lua" => "return require('./constant1')",
                "src/main.lua" => "local constant = require('./constant1.lua')\nlocal value = require('./value1.lua')",
                ".darklua.json" => DARKLUA_BUNDLE_ONLY_CONFIG,
            );

            process_main_with_error(&resources, "two_different_direct_cycles");
        }
    }
}
