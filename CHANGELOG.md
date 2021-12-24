# Changelog

## Unreleased

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
