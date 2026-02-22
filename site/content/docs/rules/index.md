---
title: Rules
description: Define which rules are used for processing
group: Configuration
order: 2
---

Rules are what transform Lua code. When processing code with darklua, it is essentially going through a given list of rules one by one, each one transforming code and feeding it to the next rule.

**Important note:** one thing to understand is that the ordering of these rules sometimes have importance! For example, you should inject a value before trying to compute expressions statically, or optimize if branches out.

Information on the built-in rules and their configuration properties can be found [here](/docs/rules-reference).

## Rule Format

Rules can be written in two different formats: the shortest format consists of simply providing the rule name. When using this format, any parameters the rule have will use its default value.

For example, the `remove_empty_do` rule does not have any parameters, so it can be specified in the configuration file as:

```json5
{
  rules: ["remove_empty_do"],
}
```

If a rule can be configured, the object format can be used to override each parameter. The rule name **must** be specified with a field named `rule`. Then, simply enumerate the property name associated with its value.

For example, the `rename_variables` rule can optionally rename the function names. To do that, the rule must be written using the object format and by defining the `include_functions` field.

```json5
{
  rule: "rename_variables",
  include_functions: true,
}
```

## Rule Filtering

When using the object format, a rule can be limited to run only for specific files using `apply_to_files` and `skip_files`. Both field use this [glob patterns implementation](https://github.com/olson-sean-k/wax/blob/master/README.md#patterns). They can be provided as a single pattern string or an array of patterns.

- `apply_to_files`: The rule runs only on files whose path matches at least one pattern. If set and the file does not match, this rule is skipped for that file.
- `skip_files`: The rule is skipped for any file whose path matches any pattern. This is **applied after** `apply_to_files`, so you can combine both.

These apply per rule: you can run one rule on all files and another only on `src/**/*.lua`, or skip a rule for test files.

```json5
rules: [
  {
    rule: "remove_comments",
    // skip this rule for anything that's in a 'vendor' folder
    skip_files: "**/vendor/**",
  },
  {
    rule: "rename_variables",
    // apply only this rule for files under the 'src' folder
    apply_to_files: ["src/**/*.lua"],
    include_functions: false,
  },
]
```
