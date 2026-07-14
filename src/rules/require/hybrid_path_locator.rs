use std::path::{Path, PathBuf};

use crate::{
    nodes::FunctionCall,
    rules::{
        require::{LuauPathLocator, PathLocator, RequirePathLocator, RobloxPathLocator},
        SingularRequireMode,
    },
    Resources,
};

#[derive(Clone, Debug)]
pub enum SingularPathLocator<'b, 'resources> {
    Path(RequirePathLocator<'b, 'resources>),
    Luau(LuauPathLocator<'b, 'resources>),
    Roblox(RobloxPathLocator<'b, 'resources>),
}

impl<'b, 'resources> SingularPathLocator<'b, 'resources> {
    fn from(
        value: &SingularRequireMode,
        extra_module_relative_location: &'b Path,
        resources: &'resources Resources,
    ) -> Self {
        match value {
            SingularRequireMode::Path(path_require_mode) => Self::Path(RequirePathLocator::new(
                path_require_mode.clone(),
                extra_module_relative_location,
                resources,
            )),
            SingularRequireMode::Luau(luau_require_mode) => Self::Luau(LuauPathLocator::new(
                luau_require_mode.clone(),
                extra_module_relative_location,
                resources,
            )),
            SingularRequireMode::Roblox(roblox_require_mode) => {
                Self::Roblox(RobloxPathLocator::new(
                    roblox_require_mode.clone(),
                    extra_module_relative_location,
                    resources,
                ))
            }
        }
    }
}

impl PathLocator for SingularPathLocator<'_, '_> {
    fn match_path_require_call(
        &self,
        call: &FunctionCall,
        source: &Path,
    ) -> Option<(PathBuf, SingularPathLocator<'_, '_>)> {
        match self {
            SingularPathLocator::Path(require_path_locator) => {
                require_path_locator.match_path_require_call(call, source)
            }
            SingularPathLocator::Luau(luau_path_locator) => {
                luau_path_locator.match_path_require_call(call, source)
            }
            SingularPathLocator::Roblox(roblox_path_locator) => {
                roblox_path_locator.match_path_require_call(call, source)
            }
        }
    }

    fn find_require_path(
        &self,
        path: impl Into<PathBuf>,
        source: &Path,
    ) -> Result<PathBuf, crate::DarkluaError> {
        match self {
            SingularPathLocator::Path(require_path_locator) => {
                require_path_locator.find_require_path(path, source)
            }
            SingularPathLocator::Luau(luau_path_locator) => {
                luau_path_locator.find_require_path(path, source)
            }
            SingularPathLocator::Roblox(roblox_path_locator) => {
                roblox_path_locator.find_require_path(path, source)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct HybridPathLocator<'b, 'resources> {
    path_locators: Vec<SingularPathLocator<'b, 'resources>>,
}

impl<'b, 'resources> HybridPathLocator<'b, 'resources> {
    pub(crate) fn new(
        require_modes: &Vec<SingularRequireMode>,
        extra_module_relative_location: &'b Path,
        resources: &'resources Resources,
    ) -> Self {
        let mut path_locators = Vec::new();

        for mode in require_modes {
            path_locators.push(SingularPathLocator::from(
                mode,
                extra_module_relative_location,
                resources,
            ))
        }

        Self { path_locators }
    }
}

impl PathLocator for HybridPathLocator<'_, '_> {
    fn match_path_require_call(
        &self,
        call: &FunctionCall,
        source: &Path,
    ) -> Option<(PathBuf, SingularPathLocator<'_, '_>)> {
        for locator in &self.path_locators {
            if let Some(x) = locator.match_path_require_call(call, source) {
                return Some(x);
            }
        }

        None
    }

    fn find_require_path(
        &self,
        _path: impl Into<PathBuf>,
        _source: &Path,
    ) -> Result<PathBuf, crate::DarkluaError> {
        Err(crate::DarkluaError::custom(
            "this cannot be called within this context",
        ))
    }
}
