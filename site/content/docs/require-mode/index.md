---
title: Require Mode
description: How the bundler understands require calls
group: Configuration
order: 4
---

When bundling code, darklua uses the require mode defined in the configuration file to find the source files.

Currently, there is only **one require mode available**:

- `path`: call require using file paths

  ```lua
  local Config = require("./Config")
  local Promise = require("Packages/Promise")
  ```

# Common Configuration

## Modules Identifier

When darklua bundles multiple modules into a single file, it uses a variable to store all the required modules. This parameter lets you modify that variable name if needed.

```json5
{
  bundle: {
    "require-mode": "path",
    // by default, darklua will use the following value
    "modules-identifier": "__DARKLUA_BUNDLE_MODULES",
  },
}
```

## Excludes

Provide a list of patterns to exclude certain paths from the bundle.

For example, to avoid bundling any require to paths starting with `@lune`:

```json5
{
  bundle: {
    "require-mode": "path",
    excludes: ["@lune/**"],
  },
}
```

These patterns are similar to Unix globs, but for more information about the differences and the syntax, see the [pattern library documentation](https://github.com/olson-sean-k/wax/blob/master/README.md#patterns) used by darklua.

# Path Require Mode

This require mode can be configured in the bundle part of the configuration file. For a quick overview of the configuration, see [the overview](../config/).

Once enabled, darklua will find all require calls made with strings (single or double quotes) and resolve them.

## Path Resolution

The first step consist of figuring out the head of the path or where to start looking for the resource:

- **if the path starts with `.` or `..`:** the path is considered relative to the file where the require call is made
- **if the path starts with `/`:** the path is considered like a regular absolute path
- **else:** the first component of the path is used to find a matching [source](#sources)

The next step is to resolve the tail of the path:

- **if the path has an extension:** the resource is expected exactly as is
- **else:** darklua will find the first available file based on the given path:

  1. the given path
  1. the given path with a `luau` extension
  1. the given path with a `lua` extension
  1. the given path joined with the module folder name
  1. (if the module folder name does not have an extension) the given path joined with the module folder name and a `luau` extension
  1. (if the module folder name does not have an extension) the given path joined with the module folder name and a `lua` extension

Here is a concrete example of these steps with a require to `./example`. darklua will try the following paths and find the first file:

1. `./example`
1. `./example.luau`
1. `./example.lua`
1. `./example/init`
1. `./example/init.luau`
1. `./example/init.lua`

## Module Folder Name

When requiring a folder, this mode will look into the folder for a file named by the given value of the `module-folder-name` parameter. The default value is `init`.

For example, to configure darklua to use `index.lua` (or `index.luau`) similar to what is used in JavaScript, set the parameter to `index`:

```json5
{
  bundle: {
    "require-mode": {
      name: "path",
      // folders with a `index.lua` or `index.luau` file
      // can be required
      "module-folder-name": "index",
    },
  },
}
```

## Sources

When a path do not start with `.`, `..` or `/`, their first component is used to find its associated source location. These locations can be configured with the `sources` parameter of the path require mode configuration.

Relative paths are resolved based on the configuration file location.

### Example

Given this configuration file:

```json5
{
  bundle: {
    "require-mode": {
      name: "path",
      sources: {
        pkg: "./Packages",
        // you can also map directly to a file (Lua or
        // any supported data file)
        images: "./assets/image-links.json",
      },
    },
  },
}
```

It is possible to make these require call in any file:

```lua
local Promise = require("pkg/Promise")
local images = require("images")
```

## Require Data Files as Lua

The `path` require mode is able to require data files and convert them into Lua data. All that is needed is that the file has one of the recognized extensions:

- [JSON](https://en.wikipedia.org/wiki/JSON) with `.json` or `.json5`
- [YAML](https://en.wikipedia.org/wiki/YAML) with `.yml` or `.yaml`
- [Toml](https://toml.io/en/) with `.toml`

If you would like to see a format added, feel free to submit a request using a [GitHub issue](https://github.com/seaofvoices/darklua/issues).

### JSON Example

<br/>

<compare-code left="JSON file" right="Generated Lua" left-language="json" right-language="lua">
<code>
{
  "experienceActivation_singleton": "experienceActivation",
  "experienceConfiguration_singleton": "experienceConfiguration",
  "experience_singleton": {
    "experience": {
      "assetId": 3296599132,
      "startPlaceId": 8667346609
    }
  },
  "placeConfiguration_start": "placeConfiguration",
  "placeFile_start": {
    "placeFile": {
      "version": 2
    }
  },
  "place_start": {
    "place": {
      "assetId": 8667346609
    }
  }
}
</code>

<code>
{
	experienceActivation_singleton = "experienceActivation",
	experienceConfiguration_singleton = "experienceConfiguration",
	experience_singleton = {
		experience = {
			assetId = 3296599132,
			startPlaceId = 8667346609,
		},
	},
	placeConfiguration_start = "placeConfiguration",
	placeFile_start = {
		placeFile = {
			version = 2,
		},
	},
	place_start = {
		place = {
			assetId = 8667346609,
		},
	},
}
</code>
</compare-code>

### YAML Example

<br/>

<compare-code left="YAML file" right="Generated Lua" left-language="yml" right-language="lua">
<code>
name: Tests
on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main
jobs:
  code-style:
    name: Verify code style
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Verify code format
        run: cargo fmt -- --check
</code>

<code>
{
    name = "Tests",
    on = {
        push = {
            branches = { "main" },
        },
        pull_request = {
            branches = { "main" },
        },
    },
    jobs = {
        ["code-style"] = {
            name = "Verify code style",
            ["runs-on"] = "ubuntu-latest",
            steps = {
                {
                    uses = "actions/checkout@v3",
                },
                {
                    name = "Verify code format",
                    run = "cargo fmt -- --check",
                },
            },
        },
    },
}
</code>
</compare-code>

### Toml Example

<br/>

<compare-code left="Toml file" right="Generated Lua" left-language="toml" right-language="lua">
<code>
[package]
name = "darklua"
version = "0.9.0"
edition = "2018"
readme = "README.md"
description = "Transform Lua scripts"
repository = "https://github.com/seaofvoices/darklua"
homepage = "https://darklua.com"
license = "MIT"
keywords = ["lua", "obsfucation", "minify"]
exclude = ["site"]

[badges]
github = { repository = "seaofvoices/darklua" }

[lib]
name = "darklua_core"
path = "src/lib.rs"

[[bin]]
name = "darklua"
path = "src/bin.rs"

[features]
tracing = ["dep:tracing"]

[dependencies]
clap = { version = "4.1.1", features = ["derive"] }
durationfmt = "0.1.1"
elsa = "1.7.0"
env_logger = "0.9.0"
full_moon = { version = "0.16.2", features = ["roblox"] }
json5 = "0.4"
log = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0.92"
serde_yaml = "0.9.17"
toml = "0.7.2"
tracing = { version = "0.1", optional = true }
wax = "0.5.0"

[dev-dependencies]
assert_cmd = "2.0.4"
criterion = { version = "0.4", features = ["html_reports"] }
include_dir = "0.7.3"
insta = { version = "1.29.0", features = ["json", "filters"] }
paste = "1.0"
pretty_assertions = "0.7.2"
rand = "0.7.3"
rand_distr = "0.2.2"
serde_bytes = "0.11"
tempfile = "3.5.0"
tracing-subscriber = "0.3.16"
tracing-tracy = "0.10.1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
node-sys = "0.4.2"
web-sys = { version = "0.3.60", features = ["Window", "Performance"] }

[profile.dev.package.full_moon]
opt-level = 3

[[bench]]
name = "process_bench"
harness = false

[[bench]]
name = "parse_bench"
harness = false

</code>

<code>
{
	bench = {
		{ harness = false, name = "process_bench" },
		{ harness = false, name = "parse_bench" },
	},
	bin = {
		{ name = "darklua", path = "src/bin.rs" },
	},
	badges = {
		github = { repository = "seaofvoices/darklua" },
	},
	dependencies = {
		durationfmt = "0.1.1",
		elsa = "1.7.0",
		env_logger = "0.9.0",
		json5 = "0.4",
		log = "0.4",
		serde_json = "1.0.92",
		serde_yaml = "0.9.17",
		toml = "0.7.2",
		wax = "0.5.0",
		clap = { features = { "derive" }, version = "4.1.1" },
		full_moon = { features = { "roblox" }, version = "0.16.2" },
		serde = { features = { "derive" }, version = "1.0" },
		tracing = { optional = true, version = "0.1" },
	},
	["dev-dependencies"] = {
		assert_cmd = "2.0.4",
		include_dir = "0.7.3",
		paste = "1.0",
		pretty_assertions = "0.7.2",
		rand = "0.7.3",
		rand_distr = "0.2.2",
		serde_bytes = "0.11",
		tempfile = "3.5.0",
		["tracing-subscriber"] = "0.3.16",
		["tracing-tracy"] = "0.10.1",
		criterion = { features = { "html_reports" }, version = "0.4" },
		insta = { features = { "json", "filters" }, version = "1.29.0" },
	},
	features = {
		tracing = { "dep:tracing" },
	},
	lib = { name = "darklua_core", path = "src/lib.rs" },
	package = {
		authors = { "jeparlefrancais <jeparlefrancais21@gmail.com>" },
		description = "Transform Lua scripts",
		edition = "2018",
		exclude = { "site" },
		homepage = "https://darklua.com",
		keywords = { "lua", "obsfucation", "minify" },
		license = "MIT",
		name = "darklua",
		readme = "README.md",
		repository = "https://github.com/seaofvoices/darklua",
		version = "0.9.0",
	},
	profile = {
		dev = {
			package = { full_moon = { ["opt-level"] = 3 } },
		},
	},
	target = {
		['cfg(target_arch = "wasm32")'] = {
			dependencies = {
				["node-sys"] = "0.4.2",
				["web-sys"] = { features = { "Window", "Performance" }, version = "0.3.60" },
			},
		},
	},
}
</code>
</compare-code>
