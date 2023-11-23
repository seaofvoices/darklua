---
description: Removes comments
added_in: "0.7.0"
parameters: []
examples:
  - content: "return nil -- this is a comment"
---

It is important to note that when generating code with the `dense` or `readable` generator (e.g. `darklua process src --format dense`), the comments will already be removed. The only way to retain comments is to use the `retain_lines` format and avoid this rule.
