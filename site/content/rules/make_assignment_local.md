---
description: Convert Luau `const` assignments to `local` assignments
added_in: "0.19.0"
parameters: []
examples:
  - content: "const PI = math.pi"
  - content: "const function example() end"
---

This rule converts all `const` assignments into regular `local` assignments. It is applied to variable assignments and function assignments.

**Note:** this rule is useful if you are converting Luau code into regular Lua code.
