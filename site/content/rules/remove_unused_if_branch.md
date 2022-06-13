---
description: Removes unused if branch
added_in: "0.3.1"
parameters: []
---

When a condition in a if branch (`if condition then` or `elseif condition then`) can be evaluated to a known, the if statement is modified to remove branches that become useless.

```lua
if unknown then
    return 2
elseif true then
    return 1
else
    return 0
end
```

Since the second branch is always true, the else block becomes superfluous. As such, this rule would output:

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
