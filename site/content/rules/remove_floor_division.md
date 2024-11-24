---
description: Removes floor divisions
added_in: "unreleased"
parameters: []
examples:
  - content: "return variable // divider"
  - content: "variable //= 5"
---

This rule removes all usage of the floor division operator (`//`). It replaces those operations with a regular division (`/`) followed by a `math.floor()` call.