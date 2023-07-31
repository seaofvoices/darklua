use darklua_core::{rules::Rule, Options, Resources};

use crate::utils;

use super::memory_resources;

test_rule!(
    convert_path_require_to_roblox,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
            rule: 'convert_require',
            current: 'path',
            target: { name: 'roblox', indexing_style: 'find_first_child' },
        }"#
    ).unwrap(),
    resources = memory_resources!(
        "src/test/init.lua" => "return nil",
        "src/test/init.luau" => "return nil",
        "src/test/module.lua" => "return nil",
        "src/test/module.luau" => "return nil",
        "src/test/folder/lib.lua" => "return nil",
        "src/sub/lib.lua" => "return nil",
        "src/format.lua" => "return nil",
        "project.lua" => "return nil",
    ),
    test_file_name = "src/test/runner.lua",
    sibling_module("local module = require('./module.lua')")
        => "local module = require(script.Parent:FindFirstChild('module'))",
    sibling_luau_module("local module = require('./module.luau')")
        => "local module = require(script.Parent:FindFirstChild('module'))",
    sibling_folder_module("local module = require('./module')")
        => "local module = require(script.Parent:FindFirstChild('module'))",
    sibling_module_is_init_file_with_extension("local module = require('./init.lua')")
        => "local module = require(script.Parent)",
    sibling_module_is_init_file_with_luau_extension("local module = require('./init.luau')")
        => "local module = require(script.Parent)",
    module_nested_in_sibling_folder("local module = require('./folder/lib.lua')")
        => "local module = require(script.Parent:FindFirstChild('folder'):FindFirstChild('lib'))",
    module_in_parent("local module = require('../format.lua')")
        => "local module = require(script.Parent.Parent:FindFirstChild('format'))",
    module_nested_in_folder_from_parent("local module = require('../sub/lib.lua')")
        => "local module = require(script.Parent.Parent:FindFirstChild('sub'):FindFirstChild('lib'))",
    module_in_double_parent("local module = require('../../project.lua')")
        => "local module = require(script.Parent.Parent.Parent:FindFirstChild('project'))",
    module_in_parent_with_current_dir("local module = require('.././format.lua')")
        => "local module = require(script.Parent.Parent:FindFirstChild('format'))",
);

test_rule!(
    convert_path_require_to_roblox_with_wait_for_child,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
            rule: 'convert_require',
            current: 'path',
            target: { name: 'roblox', indexing_style: 'wait_for_child' },
        }"#
    ).unwrap(),
    resources = memory_resources!(
        "src/test/init.lua" => "return nil",
        "src/test/init.luau" => "return nil",
        "src/test/module.lua" => "return nil",
        "src/test/module.luau" => "return nil",
        "src/test/folder/lib.lua" => "return nil",
        "src/sub/lib.lua" => "return nil",
        "src/format.lua" => "return nil",
        "project.lua" => "return nil",
    ),
    test_file_name = "src/test/runner.lua",
    sibling_module("local module = require('./module.lua')")
        => "local module = require(script.Parent:WaitForChild('module'))",
    sibling_luau_module("local module = require('./module.luau')")
        => "local module = require(script.Parent:WaitForChild('module'))",
    sibling_folder_module("local module = require('./module')")
        => "local module = require(script.Parent:WaitForChild('module'))",
    sibling_module_is_init_file_with_extension("local module = require('./init.lua')")
        => "local module = require(script.Parent)",
    sibling_module_is_init_file_with_luau_extension("local module = require('./init.luau')")
        => "local module = require(script.Parent)",
    module_nested_in_sibling_folder("local module = require('./folder/lib.lua')")
        => "local module = require(script.Parent:WaitForChild('folder'):WaitForChild('lib'))",
    module_in_parent("local module = require('../format.lua')")
        => "local module = require(script.Parent.Parent:WaitForChild('format'))",
    module_nested_in_folder_from_parent("local module = require('../sub/lib.lua')")
        => "local module = require(script.Parent.Parent:WaitForChild('sub'):WaitForChild('lib'))",
    module_in_double_parent("local module = require('../../project.lua')")
        => "local module = require(script.Parent.Parent.Parent:WaitForChild('project'))",
    module_in_parent_with_current_dir("local module = require('.././format.lua')")
        => "local module = require(script.Parent.Parent:WaitForChild('format'))",
);

test_rule!(
    convert_path_require_to_roblox_with_property_index,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
            rule: 'convert_require',
            current: 'path',
            target: { name: 'roblox', indexing_style: 'property' },
        }"#
    ).unwrap(),
    resources = memory_resources!(
        "src/test/init.lua" => "return nil",
        "src/test/init.luau" => "return nil",
        "src/test/module.lua" => "return nil",
        "src/test/module.luau" => "return nil",
        "src/test/folder/lib.lua" => "return nil",
        "src/sub/lib.lua" => "return nil",
        "src/format.lua" => "return nil",
        "project.lua" => "return nil",
        // specific to property index style tests
        "src/test/while.lua" => "return nil",
        "src/test/a module.lua" => "return nil",
    ),
    test_file_name = "src/test/runner.lua",
    sibling_module("local module = require('./module.lua')")
        => "local module = require(script.Parent.module)",
    sibling_luau_module("local module = require('./module.luau')")
        => "local module = require(script.Parent.module)",
    sibling_folder_module("local module = require('./module')")
        => "local module = require(script.Parent.module)",
    sibling_module_is_init_file_with_extension("local module = require('./init.lua')")
        => "local module = require(script.Parent)",
    sibling_module_is_init_file_with_luau_extension("local module = require('./init.luau')")
        => "local module = require(script.Parent)",
    module_nested_in_sibling_folder("local module = require('./folder/lib.lua')")
        => "local module = require(script.Parent.folder.lib)",
    module_in_parent("local module = require('../format.lua')")
        => "local module = require(script.Parent.Parent.format)",
    module_nested_in_folder_from_parent("local module = require('../sub/lib.lua')")
        => "local module = require(script.Parent.Parent.sub.lib)",
    module_in_double_parent("local module = require('../../project.lua')")
        => "local module = require(script.Parent.Parent.Parent.project)",
    module_in_parent_with_current_dir("local module = require('.././format.lua')")
        => "local module = require(script.Parent.Parent.format)",
    // specific to property index style
    sibling_module_with_keyword_name("local module = require('./while.lua')")
        => "local module = require(script.Parent['while'])",
    sibling_module_with_name_with_space("local module = require('./a module.lua')")
        => "local module = require(script.Parent['a module'])",
);

fn process_file(resources: &Resources, file_name: &str) -> String {
    darklua_core::process(resources, Options::new(file_name))
        .result()
        .unwrap();

    resources.get(file_name).unwrap()
}

fn expect_file_process(resources: &Resources, file_name: &str, expect_content: &str) {
    pretty_assertions::assert_eq!(process_file(resources, file_name), expect_content);
}

fn snapshot_file_process(resources: &Resources, file_name: &str, snapshot_name: &str) {
    insta::assert_snapshot!(
        snapshot_name,
        process_file(resources, file_name),
        &format!("process `tests/test_cases/sourcemap/{}`", file_name)
    );
}

const CONVERT_PATH_TO_ROBLOX_DEFAULT_CONFIG: &str =
    "{ rules: [{ rule: 'convert_require', current: 'path', target: 'roblox' }], generator: \"retain_lines\" }";

#[test]
fn convert_sibling_module_from_init_module() {
    let resources = memory_resources!(
        "src/init.lua" => "local value = require('./value.lua')",
        "src/value.lua" => "return nil",
        ".darklua.json" => CONVERT_PATH_TO_ROBLOX_DEFAULT_CONFIG,
    );
    expect_file_process(
        &resources,
        "src/init.lua",
        "local value = require(script:FindFirstChild('value'))",
    );
}

#[test]
fn convert_sibling_init_module_from_init_module() {
    let resources = memory_resources!(
        "src/init.lua" => "local value = require('./folder/init.lua')",
        "src/folder/init.lua" => "return nil",
        ".darklua.json" => CONVERT_PATH_TO_ROBLOX_DEFAULT_CONFIG,
    );
    expect_file_process(
        &resources,
        "src/init.lua",
        "local value = require(script:FindFirstChild('folder'))",
    );
}

#[test]
fn convert_parent_init_module_from_init_module() {
    let resources = memory_resources!(
        "src/module/init.lua" => "local value = require('../init.lua')",
        "src/init.lua" => "return nil",
        ".darklua.json" => CONVERT_PATH_TO_ROBLOX_DEFAULT_CONFIG,
    );
    expect_file_process(
        &resources,
        "src/module/init.lua",
        "local value = require(script.Parent)",
    );
}

mod sourcemap {
    use super::*;

    const CONVERT_PATH_TO_ROJO_SOURCEMAP_CONFIG: &str = r#"{
        generator: 'retain_lines',
        rules: [
            {
                rule: 'convert_require',
                current: {
                    name: 'path',
                    sources: {
                        '@pkg': './Packages'
                    }
                },
                target: {
                    name: 'roblox',
                    rojo_sourcemap: './sourcemap.json',
                }
            }
        ]
    }"#;

    fn get_resources_for_sourcemap(datamodel_case: bool) -> Resources {
        memory_resources!(
            "src/init.lua" => include_str!("../test_cases/sourcemap/src/init.lua"),
            "src/a.lua" => include_str!("../test_cases/sourcemap/src/a.lua"),
            "src/b.lua" => include_str!("../test_cases/sourcemap/src/b.lua"),
            "src/c.lua" => include_str!("../test_cases/sourcemap/src/c.lua"),

            "src/d/init.lua" => include_str!("../test_cases/sourcemap/src/d/init.lua"),
            "src/d/d1.lua" => include_str!("../test_cases/sourcemap/src/d/d1.lua"),
            "src/d/d2.lua" => include_str!("../test_cases/sourcemap/src/d/d2.lua"),

            "Packages/Package1/init.lua" => include_str!("../test_cases/sourcemap/Packages/Package1/init.lua"),
            "Packages/Package1/value.lua" => include_str!("../test_cases/sourcemap/Packages/Package1/value.lua"),

            "main.server.lua" => include_str!("../test_cases/sourcemap/main.server.lua"),

            ".darklua.json" => CONVERT_PATH_TO_ROJO_SOURCEMAP_CONFIG,
            "sourcemap.json" => if datamodel_case {
                include_str!("../test_cases/sourcemap/place-sourcemap.json")
            } else {
                include_str!("../test_cases/sourcemap/sourcemap.json")
            },
        )
    }

    #[test]
    fn invalid_sourcemap() {
        let resources = memory_resources!(
            "src/init.lua" => "return nil",
            ".darklua.json" => CONVERT_PATH_TO_ROJO_SOURCEMAP_CONFIG,
            "sourcemap.json" => "",
        );
        utils::snapshot_file_process_file_errors(&resources, "src/init.lua", "invalid_sourcemap")
    }

    #[test]
    fn convert_sibling_module_from_init_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(false),
            "src/d/init.lua",
            "convert_sibling_module_from_init_module",
        );
    }

    #[test]
    fn in_datamodel_convert_sibling_module_from_init_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(true),
            "src/d/init.lua",
            "convert_sibling_module_from_init_module",
        );
    }

    #[test]
    fn convert_module_from_child_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(false),
            "src/d/d2.lua",
            "convert_module_from_child_module",
        );
    }

    #[test]
    fn in_datamodel_convert_module_from_child_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(true),
            "src/d/d2.lua",
            "convert_module_from_child_module",
        );
    }

    #[test]
    fn convert_multiple_sibling_modules_from_root_init_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(false),
            "src/init.lua",
            "convert_multiple_sibling_modules_from_root_init_module",
        );
    }

    #[test]
    fn in_datamodel_convert_multiple_sibling_modules_from_root_init_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(true),
            "src/init.lua",
            "convert_multiple_sibling_modules_from_root_init_module",
        );
    }

    #[test]
    fn convert_sibling_module_from_sibling_module() {
        expect_file_process(
            &get_resources_for_sourcemap(false),
            "src/b.lua",
            "local a = require(script.Parent:FindFirstChild('a'))\n\nreturn a\n",
        );
    }

    #[test]
    fn in_datamodel_convert_sibling_module_from_sibling_module() {
        expect_file_process(
            &get_resources_for_sourcemap(true),
            "src/b.lua",
            "local a = require(script.Parent:FindFirstChild('a'))\n\nreturn a\n",
        );
    }

    #[test]
    fn convert_package_module_from_nested_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(false),
            "src/d/d1.lua",
            "convert_package_module_from_nested_module",
        );
    }

    #[test]
    fn in_datamodel_convert_package_module_from_nested_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(true),
            "src/d/d1.lua",
            "convert_package_module_from_nested_module",
        );
    }

    #[test]
    fn convert_nested_package_module_from_sibling_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(false),
            "Packages/Package1/init.lua",
            "convert_nested_package_module_from_sibling_module",
        );
    }

    #[test]
    fn in_datamodel_convert_nested_package_module_from_sibling_module() {
        snapshot_file_process(
            &get_resources_for_sourcemap(true),
            "Packages/Package1/init.lua",
            "convert_nested_package_module_from_sibling_module",
        );
    }

    // following tests are only on the DataModel sourcemap case
    #[test]
    fn in_datamodel_convert_module_require_across_service_instance() {
        snapshot_file_process(
            &get_resources_for_sourcemap(true),
            "main.server.lua",
            "convert_module_require_across_service_instance",
        );
    }
}
