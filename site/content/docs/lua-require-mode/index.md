---
title: Lua Require Mode
description: How lua (and darklua) understands require calls
group: Configuration
order: 6
---

This require mode is used for lua programs outside of the Luau ecosystem, including PUC lua, luajit, and most other plain lua
implementations that include a require function.
For more information, please refer to the documentation for [package.searchpath at lua.org](https://lua.org/manual/5.5/manual.html#pdf-package.searchpath).

## Bundling Support

It can be configured in the **bundle** part of the configuration file. For a quick overview of the bundling configuration, see [the documentation page](../bundle/).

## Configuration

The lua require mode uses the environment variable `LUA_PATH` by default as the search path,
but it can be configured to either use a different environment variable, just use the default search path, or
to use a string from the configuration file.
Lua's require does not support aliases or hierarchy beyond the filesystem, so there is no configuration for them.
Note that despite lua's require not supporting embedding text, JSON, TOML, and YAML files, you may do so with darklua,
as long as it is present in the search string.

### Example
```json5
{
  bundle: {
    require_mode: {
      name: "lua",
      env: "DARKLUA_PATH",
      // Path takes priority over env when provided. In the search string,
      // order is important.
      path: "./?.luau;./?/init.luau;./res/?.json;./res/?.toml"
    }
  }
}
```
