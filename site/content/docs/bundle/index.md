---
title: Bundling
description: How to bundle Lua code
group: Guides
order: 3
---

Darklua is capable of bundling Lua code: it will start from a given file and attempt to merge every require into a single file.

**Warning:** it is important that each module do not have any side effects at require-time, as the order of those side effects may not be preserved in the bundled code.

The process command will bundle Lua code when defined in the configuration file. Defining the `bundle` field will set up darklua to bundle code. The following minimal configuration will bundle code using path requires:

```json5
{
  bundle: {
    "require-mode": "path",
  },
}
```

For more information about how to configure the bundling process, take a look at the [require mode configuration](../require-mode/).

# Process Command

To bundle code, use the process command. Provide the entry point that you would like to bundle from and the second argument is the output location:

```
darklua process entry-point.lua bundled.lua
```

Given the `entry-point.lua`, darklua will recursively follow the requires and inline the code into a single `bundled.lua` file.
