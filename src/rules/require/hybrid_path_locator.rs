use std::path::{Path, PathBuf};

use crate::{Resources, nodes::FunctionCall, rules::{SingularRequireMode, require::{LuauPathLocator, PathLocator, RequirePathLocator, RobloxPathLocator}}};

#[derive(Debug)]
pub enum SingularPathLocator<'a, 'b, 'c> {
    Path(RequirePathLocator<'a, 'b, 'c>),
    Luau(LuauPathLocator<'a, 'b, 'c>),
    Roblox(RobloxPathLocator<'a, 'b, 'c>),
}

impl<'a, 'b, 'c> SingularPathLocator<'a, 'b, 'c>  {
    fn from(
        value: &'a SingularRequireMode,
        extra_module_relative_location: &'b Path,
        resources: &'c Resources,
    ) -> Self {
        match value {
            SingularRequireMode::Path(path_require_mode) => Self::Path(RequirePathLocator::new(
                &path_require_mode,
                extra_module_relative_location,
                resources,
            )),
            SingularRequireMode::Luau(luau_require_mode) => Self::Luau(LuauPathLocator::new(
                &luau_require_mode,
                extra_module_relative_location,
                resources,
            )),
            SingularRequireMode::Roblox(roblox_require_mode) => Self::Roblox(RobloxPathLocator::new(
                &roblox_require_mode,
                extra_module_relative_location,
                resources,
            )),
        }
    }
}

impl PathLocator for SingularPathLocator<'_, '_, '_> {
    fn match_path_require_call(&self, call: &FunctionCall, source: &Path) -> Option<(PathBuf, SingularPathLocator<'_, '_, '_>)> {
        match self {
            SingularPathLocator::Path(require_path_locator) => require_path_locator.match_path_require_call(call, source),
            SingularPathLocator::Luau(luau_path_locator) => luau_path_locator.match_path_require_call(call, source),
            SingularPathLocator::Roblox(roblox_path_locator) => roblox_path_locator.match_path_require_call(call, source),
        }    
    }

    fn find_require_path(
        &self,
        path: impl Into<PathBuf>,
        source: &Path,
    ) -> Result<PathBuf, crate::DarkluaError> {
        match self {
            SingularPathLocator::Path(require_path_locator) => require_path_locator.find_require_path(path, source),
            SingularPathLocator::Luau(luau_path_locator) => luau_path_locator.find_require_path(path, source),
            SingularPathLocator::Roblox(roblox_path_locator) => roblox_path_locator.find_require_path(path, source),
        }
    }
}

#[derive(Debug)]
pub(crate) struct HybridPathLocator<'a, 'b, 'resources> {
    path_locators: Vec<SingularPathLocator<'a, 'b, 'resources>>,
}

impl<'a, 'b, 'c> HybridPathLocator<'a, 'b, 'c> {
    pub(crate) fn new(
        require_modes: &'a Vec<SingularRequireMode>,
        extra_module_relative_location: &'b Path,
        resources: &'c Resources,
    ) -> Self {
        let mut path_locators = Vec::new();

        for mode in require_modes {
            path_locators.push(SingularPathLocator::from(&mode, extra_module_relative_location, resources))
        }

        Self { path_locators }
    }
}

impl PathLocator for HybridPathLocator<'_, '_, '_> {
    fn match_path_require_call(&self, call: &FunctionCall, source: &Path) -> Option<(PathBuf, SingularPathLocator<'_, '_, '_>)> {
        for locator in &self.path_locators {
            if let Some(x) = locator.match_path_require_call(call, source) {
                return Some(x)
            }
        }

        None
    }

    fn find_require_path(
            &self,
            _path: impl Into<PathBuf>,
            _source: &Path,
        ) -> Result<PathBuf, crate::DarkluaError> {
        Err(crate::DarkluaError::custom("this cannot be called within this context"))
    }
}
