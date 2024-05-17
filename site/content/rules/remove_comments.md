---
description: Removes comments
added_in: "0.7.0"
parameters:
  - name: except
    added_in: "unreleased"
    type: string array
    description: A list of regex that
examples:
  - content: "return nil -- this is a comment"
---

It is important to note that when generating code with the `dense` or `readable` generator (e.g. `darklua process src --format dense`), the comments will already be removed. The only way to retain comments is to use the `retain_lines` format and avoid this rule.

The `except` parameter is useful to avoid removing specific comments like `--!native` (which trigger native compilation of modules when using Luau on Roblox). For example, to avoid removing all comments starting with `--!`:

```json5
{
  rule: 'remove_comments',
  except: ['^--!']
}
```
