# RULES

You can find the available rules and their properties here. The default rule stack is:

  - [Remove empty do statements](#remove-empty-do-statements)
  - [Remove method definitions](#remove-method-definitions)
  - [Rename variables](#rename-variables)

---

## Remove empty do statements
```remove_empty_do```

This rule does not have any properties.

### Examples
```json5
{
    rule: 'remove_empty_do',
}
```

---

## Remove method definitions
```remove_method_definition```

Functions defined using the method syntax (with a `:`) will be replaced with their field like syntax. So the following Lua code:

```lua
local Car = {}

function Car:move(distance)
    self.position = self.position + distance
end
```

Will produce the following code:

```lua
local Car = {}

function Car.move(self, distance)
    self.position = self.position + distance
end
```

### Examples
```json5
{
    rule: 'remove_method_definition',
}
```

---

## Rename variables
```rename_variables```

Rename all declared variables and function parameters.

### Examples
The default rule configuration.
```json5
{
    rule: 'rename_variables',
    globals: ['$default'],
}
```
The configuration for Roblox Lua.
```json5
{
    rule: 'rename_variables',
    globals: ['$default', '$roblox'],
}
```
A configuration to avoid all identifiers from the default group and the identifier `a` and `b`.
```json5
{
    rule: 'rename_variables',
    globals: ['$default', 'a', 'b'],
}
```

### Property
| name | type | default | description |
| --- | --- | --- | --- |
| globals | string array | `['$default']` | a list of identifiers to avoid |

The `globals` property have special values that can be use to group multiple values together. They start with an `$` character.

#### `$default`
The default standard globals in Lua
```
arg, assert, collectgarbage, coroutine, debug, dofile, error, gcinfo, getfenv, getmetatable, io, ipairs, load, loadfile, loadstring, math, module, newproxy, next, os, package, pairs, pcall, print, rawequal, rawget, rawset, require, select, setfenv, setmetatable, string, table, tonumber, tostring, type, unpack, xpcall, _G, _VERSION
```

#### `$roblox`
The globals from Roblox Lua
```
Axes, bit32, BrickColor, CellId, ColorSequence, ColorSequenceKeypoint, Color3, CFrame, DateTime, DebuggerManager, delay, DockWidgetPluginGuiInfo, elapsedTime, Enum, Faces, Instance, LoadLibrary, game, NumberRange, NumberSequence, NumberSequenceKeypoint, PathWaypoint, PhysicalProperties, plugin, PluginDrag, PluginManager, printidentity, Random, Ray, RaycastParams, Rect, Region3, Region3int16, script, settings, shared, stats, spawn, tick, time, TweenInfo, typeof, UDim, UDim2, UserSettings, utf8, Vector2, Vector2int16, Vector3, Vector3int16, version, wait, warn, workspace, ypcall
```
