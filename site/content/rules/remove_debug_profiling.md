---
description: Removes call to debug.profilebegin and debug.profileend
added_in: "unreleased"
parameters:
  - name: preserve_arguments_side_effects
    type: boolean
    description: Defines how darklua converts the interpolated strings into `string.format` calls. The "string" strategy will make the rule use the `%s` specifier and the "tostring" strategy will use the `%*` specifier.
    default: true
examples:
  - content: |
      debug.profilebegin('function name')
      performUpdate()
      debug.profileend()
---

This rule removes all function calls to [`debug.profilebegin`](https://create.roblox.com/docs/reference/engine/libraries/debug#profilebegin) and [`debug.profileend`](https://create.roblox.com/docs/reference/engine/libraries/debug#profileend).
