use std::str::FromStr;

use schemars::schema::Schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rules::{require::PathRequireMode, RuleProcessResult};
use crate::utils::schema;
use crate::{nodes::Block, rules::Context};

use super::{path_require_mode, BundleOptions};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "name")]
pub enum BundleRequireMode {
    Path(PathRequireMode),
}

impl JsonSchema for BundleRequireMode {
    fn schema_name() -> String {
        "BundleRequireMode".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        let mut path_require_mode = gen.subschema_for::<PathRequireMode>();

        match &mut path_require_mode {
            Schema::Object(object) => {
                if let Some(reference) = &object.reference {
                    let definitions_path = &gen.settings().definitions_path;

                    if reference.starts_with(definitions_path) {
                        let name = &reference[definitions_path.len()..];
                        if let Some(Schema::Object(definition)) =
                            gen.definitions_mut().get_mut(name)
                        {
                            definition
                                .object()
                                .properties
                                .insert("name".to_owned(), schema::string_literal("path"));
                            definition.object().required.insert("name".to_owned());
                        }
                    }
                }
            }
            Schema::Bool(_) => {}
        }

        schema::one_of(vec![schema::string_literal("path"), path_require_mode])
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
        }
    }
}
