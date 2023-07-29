---
description: Convert require calls from one environment to another
added_in: "unreleased"
parameters:
  - name: current
    required: true
    type: require-mode
    description: The require mode used in the input code
  - name: target
    required: true
    type: require-mode
    description: The require mode used to generate the new require calls
examples: []
---

This rule is particularly useful if you are writing Lua code that needs to be portable to Roblox, as you can automatically convert requires by file path to Roblox instances.

Right now, the current and target require modes have certain restrictions:

- current: can only be the `path` require mode
- target: can only be the `roblox` require mode

## Configuration Overview

Here is an overview of the rule configuration format:

```json5
{
  rule: "convert_require",
  current: {
    name: "path",

    // optional (defaults to 'init')
    "module-folder-name": "init",

    // optional
    sources: {
      pkg: "./Packages",
    },
  },
  target: {
    name: "roblox",

    // optional
    "rojo-sourcemap": "./path-to/sourcemap.json",

    // optional (defaults to 'find-first-child')
    "indexing-style": "find-first-child", // 'wait-for-child' or 'property'
  },
}
```

For more information about how to configure each of require mode, visit the [path require mode documentation](/docs/path-require-mode/) and the [roblox require mode documentation](/docs/roblox-require-mode/).
