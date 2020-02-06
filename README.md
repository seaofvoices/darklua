[![pipeline status](https://gitlab.com/jeparlefrancais/darklua/badges/master/pipeline.svg)](https://gitlab.com/jeparlefrancais/darklua/commits/master)

# darklua

Obfuscate Lua 5.1 scripts.


# Usage

The following section will detail the different commands of darklua. You can also get a list of the available commands and options using the command line tool itself, simply run:
```
darklua help
```
To get help on a specific command, simply append `--help` (or `-h`) to the command name. For example, to get help on the `minify` command:
```
darklua minify --help
```

## Minify

This command reads Lua code and only reformat it in a more compact way. The input path can be a file or directory. Given a directory, darklua will find all Lua files under that directory and output them following the same hierarchy.

```
darklua minify <input-path> <output-path>

optional arguments:
  -c, --column-span <int> : default to 80
  Amount of characters before the code is wrapped into a new line
```

### Example

If you have a `src` folder that contains a bunch of Lua scripts (files ending with `.lua`), you can generate the minified version of these scripts into a new folder called `minified-src` using the following command:

```
darklua minify src minified-src
```

To specify the column-span argument, simply run:

```
darklua minify src minified-src --column-span 120
# or the shorter version:
darklua minify src minified-src -c 120
```


# Installation

darklua is a command line tool that can be installed using [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```
cargo install darklua
```

If you want to use the lastest darklua available, install it using the git url:

```
cargo install --git https://gitlab.com/jeparlefrancais/darklua.git
```


# License

darklua is available under the MIT license. See [LICENSE.txt](LICENSE.txt) for details.
