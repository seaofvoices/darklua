# RULES

You can find the available rules and their properties here. The default rule stack is:

  - [Compute expressions](#compute-expressions)
  - [Convert local functions to assignments](#convert-local-functions-to-assignments)
  - [Group local assignments](#group-local-assignments)
  - [Remove empty do statements](#remove-empty-do-statements)
  - [Remove function call parens](#remove-function-call-parens)
  - [Remove method definitions](#remove-method-definitions)
  - [Remove unused if branch](#remove-unused-if-branch)
  - [Remove unused while](#remove-unused-while)
  - [Rename variables](#rename-variables)

There are also other rules available for more processing:

  - [Inject global value](#inject-global-value)

---

## Compute expression
```compute_expression```

This rule computes expressions and replaces them with their result. An expression will not be replaced if it has any side-effects. This rule is influenced by the evaluation system of darklua. As its capacity increases, the rule will be able to compute more complex expressions. For example, if you use this rule on the following code:

```lua
return 1 + 1
```

Will produce the following code:

```lua
return 2
```

### Examples
```json5
{
    rule: 'compute_expression',
}
```

---

## Convert local functions to assignments
```convert_local_function_to_assign```

Local functions that are not recursive will be transformed to a local assignment statement. For example, if you have:

```lua
local function foo(a, b)
    return a + b
end
```

It will be converted to

```lua
local foo = function(a, b)
    return a + b
end
```

### Examples
```json5
{
    rule: 'convert_local_function_to_assign',
}
```

---

## Group local assignments
```group_local_assignment```

This rule will merge local assignments that are next to each other. For example, if you have:

```lua
local foo = 1
local bar = 2
```

Will produce the following code:

```lua
local foo, bar = 1, 2
```

The rule will not merge an assignments if it needs the previous one, since it would break the code or change the behavior. The following code would not be changed:

```lua
local foo = 1
local bar = foo
```

Since functions can return multiple values, assignments that extract more than one value will not get merged.

```lua
local foo, bar = multiple_return_values()
local baz = 0
```

### Examples
```json5
{
    rule: 'group_local_assignment',
}
```

---

## Inject global value
```inject_global_value```

This rule will find a global variable and replace it with a given value.

### Examples

To replace a global variable named `CONSTANT` with a value of `true`, you can use these settings:

```json5
{
    rule: 'inject_global_value',
    identifier: 'CONSTANT',
    value: true,
}
```

The `value` property can accept multiple types. Booleans, strings, numbers and `null` can be used.

```json5
{
    rule: 'inject_global_value',
    identifier: 'CONSTANT',
    value: 'Hello',
}
```

```json5
{
    rule: 'inject_global_value',
    identifier: 'AMOUNT',
    value: 11,
}
```

### Property
| name | type | description |
| --- | --- | --- |
| identifier | string | the identifier that will be replaced with the given value |
| value | `null`, boolean, number or string | the inlined value, if not provided  |

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

## Remove function call parens
```remove_function_call_parens```

This rule will remove parens in a function call when there is only one string or one table as arguments. It does not have any properties.

### Examples
```json5
{
    rule: 'remove_function_call_parens',
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

## Remove unused if branch
```remove_unused_if_branch```

When a condition in a if branch (`if condition then` or `elseif condition then`) can be evaluated to a known, the if statement is modified to remove branches that become useless. For example:

```lua
if unknown then
    return 2
elseif true then
    return 1
else
    return 0
end
```

Since the second branch is always true, the else block becomes useless, so this rule would output:

```lua
if unknown then
    return 2
elseif true then
    return 1
end
```

This rule can also turn if statements into do statements in certain cases:

```lua
if true then
    return 2
elseif unknown then
    return 1
else
    return 0
end
```

Since the first branch is always true, all other branches are useless, so this rule would output:

```lua
do
    return 2
end
```

This rule is influenced by the evaluation system of darklua. The more darklua can evaluate code, the better this rule can be applied.

### Examples
```json5
{
    rule: 'remove_unused_if_branch',
}
```

---

## Remove unused while
```remove_unused_while```

When a condition from a while statement can be evaluated to false and has no side effects, this rule will remove the statement. For example, the following while statement would be removed.

```lua
while "foo" == "bar" do
    -- ...
end
```

This rule is influenced by the evaluation system of darklua. The more darklua can evaluate code, the better this rule can be applied.

### Examples
```json5
{
    rule: 'remove_unused_while',
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
