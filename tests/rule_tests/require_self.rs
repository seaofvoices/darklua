// Test for @self support in require path parsing
use darklua_core::{rules::Rule, Options, Resources};

use crate::utils;

use super::memory_resources;

test_rule!(
    require_with_self_path,
    json5::from_str::<Box<dyn Rule>>(
        r#"{
            rule: 'convert_require',
            current: 'path',
            target: { name: 'roblox', indexing_style: 'find_first_child' },
        }"#
    )
    .unwrap(),
    resources = memory_resources!(
        "src/package/init.luau" => "local foo = require('@self/foo')",
        "src/package/foo.luau" => "return 42",
        ".darklua.json" => "{ \"rules\": [], \"bundle\": { \"require_mode\": \"path\" } }",
    ),
    test_file_name = "src/package/init.luau",
    require_self_foo("local foo = require(script:FindFirstChild('foo'))"),
);
