mod hybrid_path_locator;
mod luau_path_locator;
mod luau_require_mode;
mod match_require;
mod path_iterator;
mod path_locator;
mod path_require_mode;
mod roblox_path_locator;
pub(crate) mod path_utils;

use std::path::{Path, PathBuf};

pub(crate) use hybrid_path_locator::HybridPathLocator;
pub(crate) use luau_path_locator::LuauPathLocator;
pub(crate) use roblox_path_locator::RobloxPathLocator;
pub use luau_require_mode::LuauRequireMode;
pub(crate) use match_require::{is_require_call, match_path_require_call};
pub(crate) use path_locator::RequirePathLocator;
pub use path_require_mode::PathRequireMode;
pub use crate::rules::require::hybrid_path_locator::SingularPathLocator;
use crate::{DarkluaError, nodes::FunctionCall};

pub(crate) trait PathLocator {
    fn match_path_require_call(&self, call: &FunctionCall, _source: &Path) -> Option<(PathBuf, SingularPathLocator<'_, '_, '_>)>;

    fn find_require_path(
        &self,
        path: impl Into<PathBuf>,
        source: &Path,
    ) -> Result<PathBuf, DarkluaError>;
}
