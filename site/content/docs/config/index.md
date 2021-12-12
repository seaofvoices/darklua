---
title: Configuration file
description: Learn the configuration file format
group: Guides
order: 2
---

Some commands can be modified using the configuration file. darklua supports both configuration file written in the json or json5 standard file formats. When running darklua, if the directory in which the `darklua` command is executed contains a file named `.darklua.json` or `.darklua.json5`, it will automatically read the file to get the configuration values unless the `--config-path` option is given to override it.

Any missing field will be replaced with its default value.

```json5
{
  // when outputting code, darklua will wrap the code on a new line after
  // this amount of characters.
  column_span: 80,
  // put the rules that you want to execute when calling the process command.
  // If you do not provide this field, the default list of rules is going to
  // be executed.
  process: ["remove_empty_do"],
}
```

## Rule format

Rules can be written in two different formats: The shortest format consists of simply providing the rule name. Using this format, the default rule properties will be used.

```json
"remove_empty_do"
```

If a rule can be configured with properties (specific values that tweak the behavior of the rule), the table format can be used to override each properties. The rule name **must** be specified with a field named `rule`. Then, simply enumerate the property name associated with its value.

```json5
{
  rule: "the_rule_name",
  my_property_name: 50,
}
```

For example, the two following examples define two configuration files that will execute the same rule, but written in the two different formats.

```json5
{
  process: ["remove_empty_do"],
}
```

```json5
{
  process: [{ rule: "remove_empty_do" }],
}
```

Information on the built-in rules and their configuration properties can be found [here](/docs/rules).
