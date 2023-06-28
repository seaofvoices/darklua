use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::rules::RuleProcessResult;
use crate::{nodes::Block, rules::Context};

use super::{BundleOptions, PathRequireMode};

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
    pub(crate) fn process_block(
        &self,
        block: &mut Block,
        context: &Context,
        options: &BundleOptions,
    ) -> RuleProcessResult {
        match self {
            Self::Path(path_require_mode) => {
                path_require_mode.process_block(block, context, options)
            }
        }
    }
}
