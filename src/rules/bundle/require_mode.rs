use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::nodes::Block;
use crate::rules::RuleProcessResult;
use crate::{Parser, Resources};

use super::PathRequireMode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "name")]
pub enum RequireMode {
    Path(PathRequireMode),
}

impl From<PathRequireMode> for RequireMode {
    fn from(mode: PathRequireMode) -> Self {
        Self::Path(mode)
    }
}

impl FromStr for RequireMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "path" => Self::Path(Default::default()),
            _ => return Err(format!("invalid require mode `{}`", s)),
        })
    }
}

impl Default for RequireMode {
    fn default() -> Self {
        Self::Path(Default::default())
    }
}

impl RequireMode {
    pub(crate) fn process_block<'a>(
        &self,
        block: &mut Block,
        source: PathBuf,
        extra_module_relative_location: Option<&'a Path>,
        module_identifier: &'a str,
        resources: &Resources,
        parser: &'a Parser,
    ) -> RuleProcessResult {
        match self {
            Self::Path(path_require_mode) => path_require_mode.process_block(
                block,
                source,
                extra_module_relative_location,
                module_identifier,
                resources,
                parser,
            ),
        }
    }
}
