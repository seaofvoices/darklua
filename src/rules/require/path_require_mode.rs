use serde::{Deserialize, Serialize};

use crate::frontend::DarkluaResult;
use crate::nodes::{FunctionCall, StringExpression};
use crate::rules::require::match_path_require_call;
use crate::rules::Context;
use crate::DarkluaError;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::{RequirePathLocator, RequirePathLocatorMode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct PathRequireMode {
    #[serde(
        skip_serializing_if = "is_default_module_folder_name",
        default = "get_default_module_folder_name"
    )]
    module_folder_name: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    sources: HashMap<String, PathBuf>,
}

impl Default for PathRequireMode {
    fn default() -> Self {
        Self {
            module_folder_name: get_default_module_folder_name(),
            sources: Default::default(),
        }
    }
}

const DEFAULT_MODULE_FOLDER_NAME: &str = "init";

#[inline]
pub fn get_default_module_folder_name() -> String {
    DEFAULT_MODULE_FOLDER_NAME.to_owned()
}

pub fn is_default_module_folder_name(value: &String) -> bool {
    value == DEFAULT_MODULE_FOLDER_NAME
}

impl PathRequireMode {
    pub fn new(module_folder_name: impl Into<String>) -> Self {
        Self {
            module_folder_name: module_folder_name.into(),
            sources: Default::default(),
        }
    }

    pub(crate) fn find_require(
        &self,
        call: &FunctionCall,
        context: &Context,
    ) -> DarkluaResult<Option<PathBuf>> {
        if let Some(literal_path) = match_path_require_call(call) {
            let required_path =
                RequirePathLocator::new(self, context.project_location(), context.resources())
                    .find_require_path(literal_path, context.current_path())?;

            Ok(Some(required_path))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn is_module_folder_name(&self, path: &Path) -> bool {
        let expect_value = Some(self.module_folder_name.as_str());
        path.file_name().and_then(OsStr::to_str) == expect_value
            || path.file_stem().and_then(OsStr::to_str) == expect_value
    }

    pub(crate) fn generate_require(
        &self,
        path: &Path,
        _current_mode: &crate::rules::RequireMode,
        context: &Context<'_, '_, '_>,
    ) -> Result<Option<crate::nodes::Arguments>, crate::DarkluaError> {
        let mut current_path = context.current_path().to_path_buf();
        current_path.pop();
        let diff = pathdiff::diff_paths(path, &current_path).ok_or(
            DarkluaError::custom("invalid path difference").context("path require mode cannot"),
        )?;

        let mut path_str = diff
            .to_str()
            .ok_or(
                DarkluaError::custom("invalid non-UTF8 characters")
                    .context("path require mode cannot"),
            )?
            .replace("\\", "/");
        if !(path_str.starts_with("./")) {
            path_str = String::from("./") + path_str.as_str();
        }

        let string_expr = StringExpression::new(&format!("[[{path_str}]]")).map_err(|e| {
            DarkluaError::custom(format!("{e}")).context("path require mode cannot")
        })?;
        Ok(Some(crate::nodes::Arguments::String(string_expr)))
    }
}

impl RequirePathLocatorMode for PathRequireMode {
    fn get_source(&self, name: &str) -> Option<&Path> {
        self.sources.get(name).map(PathBuf::as_path)
    }
    fn module_folder_name(&self) -> &str {
        &self.module_folder_name
    }
    fn match_path_require_call(&self, call: &FunctionCall, _source: &Path) -> Option<PathBuf> {
        match_path_require_call(call)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod is_module_folder_name {
        use super::*;

        #[test]
        fn default_mode_is_false_for_regular_name() {
            let require_mode = PathRequireMode::default();

            assert!(!require_mode.is_module_folder_name(Path::new("oops.lua")));
        }

        #[test]
        fn default_mode_is_true_for_init_lua() {
            let require_mode = PathRequireMode::default();

            assert!(require_mode.is_module_folder_name(Path::new("init.lua")));
        }

        #[test]
        fn default_mode_is_true_for_init_luau() {
            let require_mode = PathRequireMode::default();

            assert!(require_mode.is_module_folder_name(Path::new("init.luau")));
        }

        #[test]
        fn default_mode_is_true_for_folder_init_lua() {
            let require_mode = PathRequireMode::default();

            assert!(require_mode.is_module_folder_name(Path::new("folder/init.lua")));
        }

        #[test]
        fn default_mode_is_true_for_folder_init_luau() {
            let require_mode = PathRequireMode::default();

            assert!(require_mode.is_module_folder_name(Path::new("folder/init.luau")));
        }
    }
}
