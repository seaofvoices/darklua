---
description: "Remove function attributes"
added_in: "unreleased"
parameters:
  - name: match
    type: string array
    description: "Attribute names matching any of the given regular expressions will be removed. When empty (default), all attributes are removed."
examples:
  - content: |
      @deprecated
      local function example()
        return process()
      end
---

The `remove_attribute` rule can remove _specific_ function attributes or _all_ function attributes. Attributes are metadata annotations that can be attached to functions using the `@` symbol (e.g., `@deprecated`, `@native`).

By default, all attributes are removed. The `match` parameter allows selective removal of only specific attributes using regular expressions. For example, to remove `@deprecated` attributes while keeping `@native`, configure the rule this way:

```json5
{
  rule: "remove_attribute",
  match: ["deprecated"],
}
```
