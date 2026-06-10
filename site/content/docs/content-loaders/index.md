---
title: Content Loaders
description: How darklua processes files
group: Configuration
order: 7
---

darklua picks a loader for each file based on its extension: Lua and Luau files are parsed, recognized data files are converted to Lua modules, and everything else is left alone.

The `loaders` field lets you override these defaults or assign a loader to any extension darklua does not recognize. It maps [glob patterns](https://github.com/olson-sean-k/wax/blob/master/README.md#patterns) to loader names. Patterns are matched **in order**, and the first one that matches a file will be selected.

Files that match no pattern fall back to the extension-based defaults (see [Reference](#reference)), and files with no recognized loader are skipped.

```json5
{
  loaders: {
    "**/*.model.json": "copy",
    "**/*.md": "string",
    "**/*.png": "buffer/base64",
  },
}
```

## Copying files

The `copy` loader writes files to the output directory unchanged. This is useful for assets that need to end up alongside your processed code, such as Rojo model files:

```json5
{
  loaders: {
    "**/*.model.json": "copy",
  },
}
```

Previously, the usual workaround was to copy these files manually (for example with `cp`) and then run darklua in-place. The `copy` loader removes that extra step.

## Processing data files

darklua can convert structured data into a Lua module that returns the data as a table. JSON, JSON Lines, TOML, and YAML are supported:

```json5
{
  loaders: {
    "**/*.json": "json",
    "**/*.jsonl": "json_lines",
    "**/*.toml": "toml",
    "**/*.yaml": "yaml",
  },
}
```

A file like `config.json` becomes `config.lua` (or `config.luau`) that returns the converted table. This already worked when bundling; loaders make it available in the regular `process` flow too.

The `json` loader parses files with a JSON5 parser, not strict JSON. That means files may include comments, trailing commas, and unquoted keys. The `json_lines` loader does the same per line: each line is parsed with the JSON5 parser and the module returns a Lua array of the parsed values. The `yaml` loader also accepts `yml` as an alias, so `"**/*.yml": "yaml"` and `"**/*.yml": "yml"` are equivalent.

## Embedding file content

These loaders read a file and create a Lua module that returns its content. They work with any file, including binary files.

- `string`: returns the content as a string.
- `buffer`: returns the content as a buffer (`buffer.fromstring(...)`).
- `bytes`: returns the content as an array of byte values.

```json5
{
  loaders: {
    "**/*.md": "string",
    "**/*.bin": "buffer",
  },
}
```

Each of these also has encoding variants that compress or encode the content before embedding it:

- `/base64`: `string/base64`, `buffer/base64`, and `bytes/base64`
- `/zstd`: `string/zstd`, `buffer/zstd`, and `bytes/zstd`
- `/gzip`: `string/gzip`, `buffer/gzip`, and `bytes/gzip`
- `/zlib`: `string/zlib`, `buffer/zlib`, and `bytes/zlib`

These variants keep binary data safe inside the generated Lua source and can reduce the size of large embedded files. Note that encoded or compressed content is **not** decoded at runtime, so your code receives the encoded value and must decode or decompress it yourself.

Unlike the data loaders (`json`, `json_lines`, `toml`, `yaml`) and `luau`, which require valid UTF-8 input, the embedding loaders accept any file content and work correctly with binary files.

## Skipping files

The `skip` loader ignores matching files. Use it to exclude files that would otherwise be picked up by a default loader:

```json5
{
  loaders: {
    "**/*.test.json": "skip",
  },
}
```

## Choosing the Lua extension

Loaders that produce a Lua module write a file with the same name but a Lua extension. By default this is `.lua`. Set `lua_extension` to control which extension is used:

```json5
{
  loaders: {
    "**/*.json": "json",
  },
  lua_extension: "luau", // or "lua"
}
```

Files that already end in `.lua` or `.luau` keep their extension.

## Requiring loaded files

Loaders that produce a Lua module work with `require` during bundling. You can `require` a JSON config, an embedded Markdown template, or any other file whose loader outputs Lua, and darklua will inline the generated module. The path in the `require` call is rewritten to use the `.lua` (or `.luau`) extension automatically, so `require("./data.json")` becomes a valid require after processing.

Files matched by `skip` or `copy` cannot be `require`d. Requiring a skipped file (or a file with no recognized loader) produces the error "configure content loaders to load this file". Requiring a copied file produces "configure content loaders to copy this file". In both cases the error message points to the relevant loader to use instead.

## Reference

Loaders available for use in the `loaders` field:

| Loader          | What it does                                                                  |
| --------------- | ----------------------------------------------------------------------------- |
| `luau`          | Parses and processes the file as Lua/Luau code.                               |
| `copy`          | Copies the file to the output unchanged.                                      |
| `skip`          | Ignores the file.                                                             |
| `string`        | Returns the content as a string.                                              |
| `string/base64` | Returns the content as a base64-encoded string.                               |
| `string/zstd`   | Returns the content as a zstd-compressed string.                              |
| `string/gzip`   | Returns the content as a gzip-compressed string.                              |
| `string/zlib`   | Returns the content as a zlib-compressed string.                              |
| `buffer`        | Returns the content as a buffer.                                              |
| `buffer/base64` | Returns the content as a base64-encoded buffer.                               |
| `buffer/zstd`   | Returns the content as a zstd-compressed buffer.                              |
| `buffer/gzip`   | Returns the content as a gzip-compressed buffer.                              |
| `buffer/zlib`   | Returns the content as a zlib-compressed buffer.                              |
| `bytes`         | Returns the content as an array of bytes.                                     |
| `bytes/base64`  | Returns the content, base64-encoded, as an array of bytes.                    |
| `bytes/zstd`    | Returns the content, zstd-compressed, as an array of bytes.                   |
| `bytes/gzip`    | Returns the content, gzip-compressed, as an array of bytes.                   |
| `bytes/zlib`    | Returns the content, zlib-compressed, as an array of bytes.                   |
| `json`          | Parses JSON data with a JSON5 parser and converts it to a Lua module.         |
| `json_lines`    | Parses each line with a JSON5 parser and converts the result to a Lua module. |
| `toml`          | Converts TOML data to a Lua module.                                           |
| `yaml`          | Converts YAML data to a Lua module.                                           |

Default loaders assigned by file extension when no pattern matches. Anything not listed here is skipped unless you assign a loader to it.

| Extension           | Loader       |
| ------------------- | ------------ |
| `.lua`, `.luau`     | `luau`       |
| `.json`, `.json5`   | `json`       |
| `.jsonl`, `.ndjson` | `json_lines` |
| `.toml`             | `toml`       |
| `.yaml`, `.yml`     | `yaml`       |
| `.txt`              | `string`     |
