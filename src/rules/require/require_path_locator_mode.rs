use std::path::{Path, PathBuf};

use crate::nodes::FunctionCall;

pub trait RequirePathLocatorMode {
    fn get_source(&self, name: &str) -> Option<&Path>;
    fn module_folder_name(&self) -> &str;
    fn require_call(&self, _call: &FunctionCall, _source: &Path) -> Option<PathBuf> {
        None
    }
    fn match_path_require_call(&self, call: &FunctionCall, source: &Path) -> Option<PathBuf>;
}