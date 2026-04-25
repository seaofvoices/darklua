use std::path::{Path, PathBuf};

use super::LuaRequireMode;
use crate::{utils, DarkluaError, Resources};

#[derive(Debug)]
pub(crate) struct LuaPathLocator<'a, 'resources> {
    lua_require_mode: &'a LuaRequireMode,
    resources: &'resources Resources,
}

impl<'a, 'b> LuaPathLocator<'a, 'b> {
    pub(crate) fn new(lua_require_mode: &'a LuaRequireMode, resources: &'b Resources) -> Self {
        Self {
            lua_require_mode,
            resources,
        }
    }
}

impl super::PathLocator for LuaPathLocator<'_, '_> {
    fn find_require_path(
        &self,
        path: impl Into<PathBuf>,
        source: &Path,
    ) -> Result<PathBuf, DarkluaError> {
        let path: String = path
            .into()
            .to_string_lossy().into_owned()
            .replace(".", std::path::MAIN_SEPARATOR_STR);
        log::trace!(
            "find require path for `{}` from `{}` (lua mode)",
            path,
            source.display()
        );
        for potential_path in self
            .lua_require_mode
            .lua_path()
            .iter()
            .map(|x| x.replace("?", &path))
        {
            let potential_pathbuf: PathBuf = potential_path.into();
            if self.resources.is_file(&potential_pathbuf)? {
                return Ok(utils::normalize_path_with_current_dir(potential_pathbuf));
            }
        }

        Err(
            DarkluaError::resource_not_found(path.clone()).context(format!(
                "tried `{}`",
                self.lua_require_mode
                    .lua_path()
                    .iter()
                    .map(|x| x.replace("?", &path))
                    .collect::<Vec<_>>()
                    .join("`, `")
            )),
        )
    }
}
