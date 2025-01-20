---
description: Inject a global variable
added_in: "0.3.5"
parameters:
  - name: identifier
    required: true
    type: string
    description: The name of the global variable
  - name: value
    type: boolean, number or string
    description: The value to inject
    default: nil
  - name: env
    added_in: "0.7.0"
    type: string
    description: An environment variable to read the value from
examples:
  - rules: "[{ rule: 'inject_global_value', identifier: 'CONSTANT', value: 'Hello' }, { rule: 'inject_global_value', identifier: 'AMOUNT', value: 11 }]"
    content: |
      if _G.AMOUNT > 10 or _G.CONSTANT ~= nil then
        --[[ ... ]]
      end
---

This rule will find a global variable and replace it with a given value. The value can be defined in the rule configuration or taken from an environment variable.

If `value` is not specified, the `env` property can be defined to read an environment variable that will be read into a string.

```json5
{
  rule: "inject_global_value",
  identifier: "GLOBAL",
  env: "SOME_VARIABLE",
}
```

This rule can be used in combination with the `remove_unused_if_branch`, `compute_expression`, and other rules, to eliminate dead branches. In addition to making your code smaller, it should make it faster (depending on how hot the code path is) since it is eliminating branch condition evaluations at client-side runtime.
