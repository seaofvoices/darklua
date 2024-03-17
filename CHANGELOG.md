# Changelog

## Unreleased

## 0.13.0

* add `convert` command to convert data files (`json`, `json5`, `yaml` and `toml`) into Lua modules ([#178](https://github.com/seaofvoices/darklua/pull/178))
* remove previously generated files between process runs in watch mode ([#177](https://github.com/seaofvoices/darklua/pull/177))
* fix `remove_compound_assignment` rule to avoid copying variable tokens ([#176](https://github.com/seaofvoices/darklua/pull/176))
* add get link button to [try-it](https://darklua.com/try-it/) page ([#175](https://github.com/seaofvoices/darklua/pull/175))
* add rule to remove unused variables (`remove_unused_variable`). Fix issue with `rename_variables` where `self` variables and some cases of variable shadowing were not correctly renamed ([#172](https://github.com/seaofvoices/darklua/pull/172))

## 0.12.1

* fix `append_text_comment` rule to support multiline comments ([#167](https://github.com/seaofvoices/darklua/pull/167))

## 0.12.0

* fix relative parent paths when working with sourcemaps ([#164](https://github.com/seaofvoices/darklua/pull/164))
* add rule to remove assertions (`remove_assertions`) ([#163](https://github.com/seaofvoices/darklua/pull/163))
* add rule to remove Roblox profiling function calls (`remove_debug_profiling`) ([#162](https://github.com/seaofvoices/darklua/pull/162))
* add rule to remove interpolated strings (`remove_interpolated_string`) ([#156](https://github.com/seaofvoices/darklua/pull/156))
* add support for floor division (`//`) operator in binary expressions ([#155](https://github.com/seaofvoices/darklua/pull/155))
* add support for Luau interpolated strings ([#94](https://github.com/seaofvoices/darklua/pull/94))
* add rule to append text comments ([#141](https://github.com/seaofvoices/darklua/pull/141))

## 0.11.3

* fix undeclared modules variable when bundling ([#151](https://github.com/seaofvoices/darklua/pull/151))

## 0.11.2

* fix bundling to handle modules with early return calls. This change also makes the bundled code preserve the module require ordering ([#147](https://github.com/seaofvoices/darklua/pull/147))
* fix bundling to avoid token reference removal errors ([#146](https://github.com/seaofvoices/darklua/pull/146))
* fix `remove_types` rule to handle type cast of expressions that could return multiple values ([#142](https://github.com/seaofvoices/darklua/pull/142))

## 0.11.1

* fix type packs, function variadic types and variadic type packs ([#137](https://github.com/seaofvoices/darklua/pull/137))
* fix generic types on function types ([#136](https://github.com/seaofvoices/darklua/pull/136))
* fix table types to allow string literal property types ([#135](https://github.com/seaofvoices/darklua/pull/135))

## 0.11.0

* fix lost comment or spacing tokens in empty ASTs ([#132](https://github.com/seaofvoices/darklua/pull/132))
* add rule to remove Luau types ([#130](https://github.com/seaofvoices/darklua/pull/130))
* add support for Luau types ([#129](https://github.com/seaofvoices/darklua/pull/129))

## 0.10.3

* add watch argument (`--watch` or `-w`) to process command to automatically re-run darklua on file changes ([#123](https://github.com/seaofvoices/darklua/pull/123))

## 0.10.2

* fix crashes when bundling code and fix `remove_spaces` and `remove_comments` rule for missing cases (if expressions and numeric for) ([#119](https://github.com/seaofvoices/darklua/pull/119))

## 0.10.1

* make rojo sourcemap paths relative to its file location ([#117](https://github.com/seaofvoices/darklua/pull/117))

## 0.10.0

* convert configuration to snake case (this renames the `retain-lines` generator to `retain_lines`) ([#114](https://github.com/seaofvoices/darklua/pull/114))
* fix dense and readable string generator to escape extended ascii using backslashes ([#111](https://github.com/seaofvoices/darklua/pull/111))
* fix number parsing to support underscore after decimal point (like `0._123`) ([#110](https://github.com/seaofvoices/darklua/pull/110))
* add rule to convert require modes (from paths to Roblox instances) ([#107](https://github.com/seaofvoices/darklua/pull/107))
* fix number parsing to support underscores before `x` in hexadecimal number and before `b` in binary numbers ([#103](https://github.com/seaofvoices/darklua/pull/103))
* add bundling with path require mode ([#97](https://github.com/seaofvoices/darklua/pull/97))
* upgrade full-moon parser to 0.18.1 ([#100](https://github.com/seaofvoices/darklua/pull/100))
* enable stacker feature on full-moon to avoid stack overflows on large input ([#109](https://github.com/seaofvoices/darklua/pull/109))

## 0.9.0

* improve `convert_index_to_field` to refactor table entries ([#88](https://github.com/seaofvoices/darklua/pull/88))
* fix `remove_nil_declaration` ([#84](https://github.com/seaofvoices/darklua/pull/84))
* upgrade CLI library (mostly changes the help messages format) ([#83](https://github.com/seaofvoices/darklua/pull/83))
* add rule to remove compount assignments ([#78](https://github.com/seaofvoices/darklua/pull/78))
* enhance `remove_unused_if_branch` to process if expressions ([#77](https://github.com/seaofvoices/darklua/pull/77))
* remove possible panic in AST parsing ([#74](https://github.com/seaofvoices/darklua/pull/74))
* fix large AST parsing issue ([#73](https://github.com/seaofvoices/darklua/pull/73))
* refactor ParserError into an opaque struct (instead of an enum) ([#71](https://github.com/seaofvoices/darklua/pull/71))
* refactor darklua frontend ([#69](https://github.com/seaofvoices/darklua/pull/69)):
  * the `--config-path` argument of the `minify` command was removed
  * the configuration file does not accept the `column_span` field anymore (use the [`generator` field](https://darklua.com/docs/generators/) instead)
  * darklua can now also read `.luau` files

## 0.8.0

* update configuration file ([!60](https://gitlab.com/seaofvoices/darklua/-/merge_requests/60))
* add rule to remove statements after a do blocks returns early ([!59](https://gitlab.com/seaofvoices/darklua/-/merge_requests/59))
* fix readable formatter to put a space after `return` keywords ([!58](https://gitlab.com/seaofvoices/darklua/-/merge_requests/58))
* fix the `remove_nil_declaration` to also pop commas correctly ([!57](https://gitlab.com/seaofvoices/darklua/-/merge_requests/57))
* fix bug where filtering statements of a block would panic when there were missing semicolons ([!56](https://gitlab.com/seaofvoices/darklua/-/merge_requests/56))
* add rule to remove unnecessary `nil` values in local assignments ([!54](https://gitlab.com/seaofvoices/darklua/-/merge_requests/54))
* enhance the `compute_expression` rule to remove left side of binary `and` or `or` expressions when they always return the right side and they don't have side effects ([!53](https://gitlab.com/seaofvoices/darklua/-/merge_requests/53))
* add support for if expressions ([!51](https://gitlab.com/seaofvoices/darklua/-/merge_requests/51))
* add function to get all valid rule names ([!46](https://gitlab.com/seaofvoices/darklua/-/merge_requests/46))

## 0.7.0

* add `include_functions` property to `rename_variables` to prevent function name renaming and remove `group_local_assignment` and `convert_local_function_to_assign` from default rules ([!44](https://gitlab.com/seaofvoices/darklua/-/merge_requests/44))
* add `env` property to `inject_global_value` to inject an environment variable value ([!43](https://gitlab.com/seaofvoices/darklua/-/merge_requests/43))
* fix command line tracing logs for planned work ([!42](https://gitlab.com/seaofvoices/darklua/-/merge_requests/42))
* fix extra space generated with retain-lines generator on field expressions ([!41](https://gitlab.com/seaofvoices/darklua/-/merge_requests/41))
* enhance the `compute_expression` rule by processing `<`, `<=`, `>` and `>=` operators ([!40](https://gitlab.com/seaofvoices/darklua/-/merge_requests/40))
* enhance the `compute_expression` rule by processing parentheses expressions ([!39](https://gitlab.com/seaofvoices/darklua/-/merge_requests/39))
* add rule to convert index expression to field expression ([!36](https://gitlab.com/seaofvoices/darklua/-/merge_requests/36))
* add logging to time processing steps ([!35](https://gitlab.com/seaofvoices/darklua/-/merge_requests/35))
* add rule to remove spaces ([!34](https://gitlab.com/seaofvoices/darklua/-/merge_requests/34))
* add rule to remove comments ([!33](https://gitlab.com/seaofvoices/darklua/-/merge_requests/33))
* fix block mutations to handle semicolon tokens ([!32](https://gitlab.com/seaofvoices/darklua/-/merge_requests/32))
* enhance the `compute_expression` rule by improving the evaluation of binary and unary expressions ([!31](https://gitlab.com/seaofvoices/darklua/-/merge_requests/31))

## 0.6.1

* enhance the `inject_global_value` rule to work also from the global table (`_G`) ([!30](https://gitlab.com/seaofvoices/darklua/-/merge_requests/30))

## 0.6.0

* add new generator that retains line numbers ([!28](https://gitlab.com/seaofvoices/darklua/-/merge_requests/28))
* add token information to Block ([!27](https://gitlab.com/seaofvoices/darklua/-/merge_requests/27))
* add computation of binary expression using concat operator ([!25](https://gitlab.com/seaofvoices/darklua/-/merge_requests/25))
* fix bugs with string generation ([!26](https://gitlab.com/seaofvoices/darklua/-/merge_requests/26))
* add token information to AST ([!24](https://gitlab.com/seaofvoices/darklua/-/merge_requests/24))

## 0.5.0

* add support for Luau syntax by switching to [full-moon](https://github.com/Kampfkarren/full-moon) for parsing ([!18](https://gitlab.com/seaofvoices/darklua/-/merge_requests/18))

## 0.4.1

* add new rule traits ([!16](https://gitlab.com/seaofvoices/darklua/-/merge_requests/16))
* fix installation using `cargo install` command ([!17](https://gitlab.com/seaofvoices/darklua/-/merge_requests/17))

## 0.4.0

* add readable code generator ([!13](https://gitlab.com/seaofvoices/darklua/-/merge_requests/13))

## 0.3.6

* add rule to compute expressions ([!12](https://gitlab.com/seaofvoices/darklua/-/merge_requests/12))

## 0.3.5

* add rule to inject a value into a global variable ([!10](https://gitlab.com/seaofvoices/darklua/-/merge_requests/10))

## 0.3.4

* add rule to remove function call parentheses ([!11](https://gitlab.com/seaofvoices/darklua/-/merge_requests/11))

## 0.3.3

* add rule to convert local functions to local assignments ([!4](https://gitlab.com/seaofvoices/darklua/-/merge_requests/4))

## 0.3.2

* add rule to group local assignment statements ([!8](https://gitlab.com/seaofvoices/darklua/-/merge_requests/8))

## 0.3.1

* add rule to remove unused if branches ([!7](https://gitlab.com/seaofvoices/darklua/-/merge_requests/7))

## 0.3.0

* fix code generation bugs ([!9](https://gitlab.com/seaofvoices/darklua/-/merge_requests/9))

## 0.2.3

* add rule to remove unused while statements ([!6](https://gitlab.com/seaofvoices/darklua/-/merge_requests/6))

## 0.2.2

* add rule to remove method definitions ([!5](https://gitlab.com/seaofvoices/darklua/-/merge_requests/5))

## 0.2.1

* add rule to rename variables ([!2](https://gitlab.com/seaofvoices/darklua/-/merge_requests/2))

## 0.2.0

* add process command and rule to remove empty do statement ([!1](https://gitlab.com/seaofvoices/darklua/-/merge_requests/1))

## 0.1.0

* minify command
