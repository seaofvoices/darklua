---
description: Convert index expression into field expressions
added_in: "0.7.0"
parameters: []
examples:
  - content: "return var['field']"
---

When an index expression is using a static string (or an expression that can be statically evaluated into a string), this rule replaces it with a field expression. For example, if you have this code:
