[![pipeline status](https://gitlab.com/jeparlefrancais/darklua/badges/master/pipeline.svg)](https://gitlab.com/jeparlefrancais/darklua/commits/master)
[![version](https://img.shields.io/crates/v/darklua)](https://crates.io/crates/darklua)
[![license](https://img.shields.io/crates/l/darklua)](LICENSE.txt)

# darklua

Transform Lua 5.1 scripts using [rules](RULES.md).


# Installation
darklua is a command line tool that can be installed using [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```
cargo install darklua
```

If you want to use the lastest darklua available, install it using the git url:

```
cargo install --git https://gitlab.com/jeparlefrancais/darklua.git
```


# Usage
The following section will detail the different commands of darklua. You can also get a list of the available commands and options using the command line tool itself, simply run:
```
darklua help
```
To get help on a specific command, simply append `--help` (or `-h`) to the command name. For example, to get help on the `minify` command:
```
darklua minify --help
```

## Process
The process command is similar to the minify command: it takes an input path and generates code at the output path. This command will apply [rules](RULES.md) (the default rules or the one given in the configuration file) to each Lua file.

```
darklua process <input-path> <output-path>

optional arguments:
  -c, --config-path <path>
  Path to a configuration file
```

### Example
If you have a `src` folder that contains a bunch of Lua scripts (files ending with `.lua`), you can process all the files with the default configuration (or with the configuration file located in the same folder where you are running the command) into a new folder called `processed-src` using the following command:

```
darklua process src processed-src
```

If a configuration file is found in the folder where the command is ran, darklua will automatically use it. If your configuration file is not named `.darklua.json` or `.darklua.json5`, or not located in the folder where you are running the command, you can specify it with the `--config-path` argument:

```
darklua process src processed-src --config-path ./path/config.json
# or the shorter version:
darklua process src processed-src -c ./path/config.json
```

## Minify
This command reads Lua code and only reformat it in a more compact way. The input path can be a file or directory. Given a directory, darklua will find all Lua files under that directory and output them following the same hierarchy.

```
darklua minify <input-path> <output-path>

optional arguments:
  -c, --config-path <path>
  Path to a configuration file
```

### Example
If you have a `src` folder that contains a bunch of Lua scripts (files ending with `.lua`), you can generate the minified version of these scripts into a new folder called `minified-src` using the following command:

```
darklua minify src minified-src
```

To specify the configuration file location, simply run:

```
darklua minify src minified-src --config-path ./path/config.json
# or the shorter version:
darklua minify src minified-src -c ./path/config.json
```


# Configuration file
Some commands can be modified using the configuration file. darklua supports both configuration file written in json or json5. When running darklua, if the folder where the command is executed contains a file named `.darklua.json` or `.darklua.json5`, it will automatically read the file to get the configuration values.

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
Rules can be written in two different format. The shortest one consist of simply providing the rule name. Using this format, the default rule properties will be used.

```json
"remove_empty_do"
```

If a rule can be configured with properties, the table format can be used to override each properties. The rule name **must** be specified with a field named `rule`. Then, simply enumerate the property name associated with its value.

```json5
{
    rule: "the_rule_name",
    property: 50
}
```

For example, the two following snippets will define two configuration files that will execute the same rule, but written in the two different formats.

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

More informations can be found for [each rule here](RULES.md).


# License

darklua is available under the MIT license. See [LICENSE.txt](LICENSE.txt) for details.
