use std::path::{Path, PathBuf};

use crate::{DarkluaError, Resources, nodes::FunctionCall, rules::{RobloxRequireMode, parse_roblox, require::{PathLocator, hybrid_path_locator::SingularPathLocator, path_iterator, path_utils::is_require_relative}}, utils};

#[derive(Clone, Debug)]
pub(crate) struct RobloxPathLocator<'a, 'b, 'resources> {
    _roblox_require_mode: &'a RobloxRequireMode,
    _extra_module_relative_location: &'b Path,
    resources: &'resources Resources,
}

impl<'a, 'b, 'c> RobloxPathLocator<'a, 'b, 'c> {
    pub(crate) fn new(
        roblox_require_mode: &'a RobloxRequireMode,
        extra_module_relative_location: &'b Path,
        resources: &'c Resources,
    ) -> Self {
        Self {
            _roblox_require_mode: roblox_require_mode,
            _extra_module_relative_location: extra_module_relative_location,
            resources,
        }
    }
}

impl PathLocator for RobloxPathLocator<'_, '_, '_> {
    fn match_path_require_call(&self, call: &FunctionCall, source: &Path) -> Option<(PathBuf, SingularPathLocator<'_, '_, '_>)> {
        parse_roblox(call, source)
            .ok()
            .flatten()
            .and_then(|x| {
                let mut source_parent = source.to_path_buf();
                source_parent.pop();
                pathdiff::diff_paths(x, source_parent).map(|x| PathBuf::from("./").join(x))
            })
            .map(|x| (x, SingularPathLocator::Roblox(self.clone())))
    }

    fn find_require_path(
            &self,
            path: impl Into<PathBuf>,
            source: &Path,
        ) -> Result<PathBuf, crate::DarkluaError> {
        let mut path: PathBuf = path.into();
        log::trace!(
            "find require path for `{}` from `{}`",
            path.display(),
            source.display()
        );

        if is_require_relative(&path) {
            let mut new_path = source.to_path_buf();
            new_path.pop();
            new_path.push(path);
            path = new_path;
        }

        let normalized_path = utils::normalize_path_with_current_dir(&path);
        for potential_path in path_iterator::find_require_paths(
            &normalized_path,
            "init",
        ) {
            if self.resources.is_file(&potential_path)? {
                return Ok(utils::normalize_path_with_current_dir(potential_path));
            }
        }

        Err(
            DarkluaError::resource_not_found(&normalized_path).context(format!(
                "tried `{}`",
                path_iterator::find_require_paths(
                    &normalized_path,
                    "init"
                )
                .map(|potential_path| potential_path.display().to_string())
                .collect::<Vec<_>>()
                .join("`, `")
            )),
        )
    }
}
