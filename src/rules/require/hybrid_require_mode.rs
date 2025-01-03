use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{nodes::FunctionCall, rules::parse_roblox};

use super::{match_path_require_call, PathRequireMode, RequirePathLocatorMode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HybridRequireMode {
    #[serde(flatten)]
    path_require_mode: PathRequireMode,
}

impl RequirePathLocatorMode for HybridRequireMode {
    fn get_source(&self, name: &str) -> Option<&Path> {
        self.path_require_mode.get_source(name)
    }
    fn module_folder_name(&self) -> &str {
        self.path_require_mode.module_folder_name()
    }
    fn match_path_require_call(&self, call: &FunctionCall, source: &Path) -> Option<PathBuf> {
        parse_roblox(call, source)
            .ok()
            .flatten()
            .and_then(|x| {
                let mut source_parent = source.to_path_buf();
                source_parent.pop();
                pathdiff::diff_paths(x, source_parent).map(|x| PathBuf::from("./").join(x))
            })
            .or(match_path_require_call(call))
    }
}
