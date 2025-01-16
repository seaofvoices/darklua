use serde::{Deserialize, Serialize};

use crate::frontend::DarkluaResult;
use crate::nodes::FunctionCall;
use crate::rules::require::match_path_require_call;
use crate::rules::Context;
use crate::utils::find_luau_configuration;
use crate::DarkluaError;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::RequirePathLocator;

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
    #[serde(default = "default_use_luau_configuration")]
    use_luau_configuration: bool,
    #[serde(skip)]
    luau_rc_aliases: Option<HashMap<String, PathBuf>>,
}

fn default_use_luau_configuration() -> bool {
    true
}

impl Default for PathRequireMode {
    fn default() -> Self {
        Self {
            module_folder_name: get_default_module_folder_name(),
            sources: Default::default(),
            use_luau_configuration: default_use_luau_configuration(),
            luau_rc_aliases: Default::default(),
        }
    }
}

const DEFAULT_MODULE_FOLDER_NAME: &str = "init";

#[inline]
fn get_default_module_folder_name() -> String {
    DEFAULT_MODULE_FOLDER_NAME.to_owned()
}

fn is_default_module_folder_name(value: &String) -> bool {
    value == DEFAULT_MODULE_FOLDER_NAME
}

impl PathRequireMode {
    pub fn new(module_folder_name: impl Into<String>) -> Self {
        Self {
            module_folder_name: module_folder_name.into(),
            sources: Default::default(),
            use_luau_configuration: default_use_luau_configuration(),
            luau_rc_aliases: Default::default(),
        }
    }

    pub(crate) fn initialize(&mut self, context: &Context) -> Result<(), DarkluaError> {
        if !self.use_luau_configuration {
            self.luau_rc_aliases.take();
            return Ok(());
        }

        if let Some(config) = find_luau_configuration(context.current_path(), context.resources())?
        {
            self.luau_rc_aliases.replace(config.aliases);
        } else {
            self.luau_rc_aliases.take();
        }

        Ok(())
    }

    pub(crate) fn module_folder_name(&self) -> &str {
        &self.module_folder_name
    }

    pub(crate) fn get_source(&self, name: &str, rel: &Path) -> Option<PathBuf> {
        self.sources
            .get(name)
            .map(|alias| rel.join(alias))
            .or_else(|| {
                self.luau_rc_aliases
                    .as_ref()
                    .and_then(|aliases| aliases.get(name))
                    .map(ToOwned::to_owned)
            })
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
        _path: &Path,
        _current_mode: &crate::rules::RequireMode,
        _context: &Context<'_, '_, '_>,
    ) -> Result<Option<crate::nodes::Arguments>, crate::DarkluaError> {
        Err(DarkluaError::custom("unsupported target require mode")
            .context("path require mode cannot be used"))
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
