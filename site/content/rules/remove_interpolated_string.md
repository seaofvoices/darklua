---
description: Removes interpolated strings (backtick strings)
added_in: "unreleased"
parameters:
  - name: strategy
    type: '"string" or "tostring"'
    description: Defines how darklua converts the interpolated strings into `string.format` calls. The "string" strategy will make the rule use the `%s` specifier and the "tostring" strategy will use the `%*` specifier.
    default: string
examples:
  - content: "return `abc`"
  - content: "return ``"
  - content: "return `+{value} (in seconds)`"
  - content: "return `Total = {#elements}`"
---

This rule removes all interpolated strings and replaces them with `string.format` calls.
