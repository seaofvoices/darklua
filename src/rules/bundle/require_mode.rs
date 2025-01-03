use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::rules::{
    require::{HybridRequireMode, PathRequireMode},
    RuleProcessResult,
};
use crate::{nodes::Block, rules::Context};

use super::{hybrid_require_mode, path_require_mode, BundleOptions};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "name")]
pub enum BundleRequireMode {
    Hybrid(HybridRequireMode),
    Path(PathRequireMode),
}

impl From<HybridRequireMode> for BundleRequireMode {
    fn from(value: HybridRequireMode) -> Self {
        Self::Hybrid(value)
    }
}

impl From<PathRequireMode> for BundleRequireMode {
    fn from(mode: PathRequireMode) -> Self {
        Self::Path(mode)
    }
}

impl FromStr for BundleRequireMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "path" => Self::Path(Default::default()),
            "hybrid" => Self::Hybrid(Default::default()),
            _ => return Err(format!("invalid require mode `{}`", s)),
        })
    }
}

impl Default for BundleRequireMode {
    fn default() -> Self {
        Self::Path(Default::default())
    }
}

impl BundleRequireMode {
    pub(crate) fn process_block(
        &self,
        block: &mut Block,
        context: &Context,
        options: &BundleOptions,
    ) -> RuleProcessResult {
        match self {
            Self::Path(path_require_mode) => {
                path_require_mode::process_block(block, context, options, path_require_mode)
            }
            Self::Hybrid(hybrid_require_mode) => {
                hybrid_require_mode::process_block(block, context, options, hybrid_require_mode)
            }
        }
    }
}
