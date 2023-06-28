mod expression_serializer;
mod path_require_mode;
mod require_mode;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::nodes::Block;
use crate::rules::{
    verify_required_properties, Context, Rule, RuleConfiguration, RuleConfigurationError,
    RuleProcessResult, RuleProperties, RulePropertyValue,
};
use crate::Parser;

pub(crate) use expression_serializer::LuaSerializerError;

pub use path_require_mode::PathRequireMode;
pub use require_mode::RequireMode;
use wax::Pattern;

pub const BUNDLER_RULE_NAME: &str = "bundler";

#[derive(Debug)]
pub(crate) struct BundleOptions {
    parser: Parser,
    extra_module_relative_location: PathBuf,
    modules_identifier: String,
    excludes: Option<wax::Any<'static>>,
}

impl BundleOptions {
    fn new<'a>(
        parser: Parser,
        extra_module_relative_location: impl Into<PathBuf>,
        modules_identifier: impl Into<String>,
        excludes: impl Iterator<Item = &'a str>,
    ) -> Self {
        let excludes: Vec<_> = excludes
            .filter_map(|exclusion| match wax::Glob::new(exclusion) {
                Ok(glob) => Some(glob.into_owned()),
                Err(err) => {
                    log::warn!(
                        "unable to create exclude matcher from `{}`: {}",
                        exclusion,
                        err.to_string()
                    );
                    None
                }
            })
            .collect();
        Self {
            parser,
            extra_module_relative_location: extra_module_relative_location.into(),
            modules_identifier: modules_identifier.into(),
            excludes: if excludes.is_empty() {
                None
            } else {
                let any_pattern = wax::any::<wax::Glob, _>(excludes)
                    .expect("exclude globs errors should be filtered and only emit a warning");
                Some(any_pattern)
            },
        }
    }

    fn parser(&self) -> &Parser {
        &self.parser
    }

    fn modules_identifier(&self) -> &str {
        &self.modules_identifier
    }

    fn extra_module_relative_location(&self) -> &Path {
        self.extra_module_relative_location.as_path()
    }

    fn is_excluded(&self, require: &Path) -> bool {
        self.excludes
            .as_ref()
            .map(|any| any.is_match(require))
            .unwrap_or(false)
    }
}

/// A rule that inlines required modules
#[derive(Debug)]
pub(crate) struct Bundler {
    require_mode: RequireMode,
    options: BundleOptions,
}

impl Bundler {
    pub(crate) fn new<'a>(
        parser: Parser,
        extra_module_relative_location: impl Into<PathBuf>,
        require_mode: RequireMode,
        excludes: impl Iterator<Item = &'a str>,
    ) -> Self {
        Self {
            require_mode,
            options: BundleOptions::new(
                parser,
                extra_module_relative_location,
                DEFAULT_MODULE_IDENTIFIER,
                excludes,
            ),
        }
    }

    pub(crate) fn with_modules_identifier(mut self, modules_identifier: impl Into<String>) -> Self {
        self.options.modules_identifier = modules_identifier.into();
        self
    }
}

impl Rule for Bundler {
    fn process(&self, block: &mut Block, context: &Context) -> RuleProcessResult {
        self.require_mode
            .process_block(block, context, &self.options)
    }
}

impl RuleConfiguration for Bundler {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_required_properties(&properties, &["require-mode"])?;

        for (key, value) in properties {
            match key.as_str() {
                "modules-identifier" => match value {
                    RulePropertyValue::String(identifier) => {
                        self.options.modules_identifier = identifier;
                    }
                    _ => return Err(RuleConfigurationError::StringExpected(key)),
                },
                "require-mode" => match value {
                    RulePropertyValue::String(require_mode) => {
                        self.require_mode =
                            RequireMode::from_str(&require_mode).map_err(|err| {
                                RuleConfigurationError::UnexpectedValue {
                                    property: "require-mode".to_owned(),
                                    message: err,
                                }
                            })?;
                    }
                    RulePropertyValue::RequireMode(require_mode) => {
                        self.require_mode = require_mode;
                    }
                    _ => return Err(RuleConfigurationError::StringExpected(key)),
                },
                _ => return Err(RuleConfigurationError::UnexpectedProperty(key)),
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        BUNDLER_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        let mut properties = RuleProperties::new();

        properties.insert(
            "require-mode".to_owned(),
            RulePropertyValue::from(&self.require_mode),
        );

        if self.options.modules_identifier != DEFAULT_MODULE_IDENTIFIER {
            properties.insert(
                "modules-identifier".to_owned(),
                RulePropertyValue::from(&self.options.modules_identifier),
            );
        }

        properties
    }
}

const DEFAULT_MODULE_IDENTIFIER: &str = "__DARKLUA_BUNDLE_MODULES";

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    fn new_rule() -> Bundler {
        Bundler::new(
            Parser::default(),
            "./modules",
            RequireMode::default(),
            std::iter::empty(),
        )
    }

    fn new_rule_with_require_mode(mode: impl Into<RequireMode>) -> Bundler {
        Bundler::new(
            Parser::default(),
            "./modules",
            mode.into(),
            std::iter::empty(),
        )
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_bundler", rule);
    }

    #[test]
    fn serialize_path_require_mode_with_custom_module_folder_name() {
        let rule: Box<dyn Rule> =
            Box::new(new_rule_with_require_mode(PathRequireMode::new("__init__")));

        assert_json_snapshot!("path_require_mode_with_custom_module_folder_name", rule);
    }

    #[test]
    fn serialize_path_require_mode_with_custom_module_folder_name_and_modules_identifier() {
        let rule: Box<dyn Rule> = Box::new(
            new_rule_with_require_mode(PathRequireMode::new("__init__"))
                .with_modules_identifier("_CUSTOM_VAR"),
        );

        assert_json_snapshot!(
            "path_require_mode_with_custom_module_folder_name_and_modules_identifier",
            rule
        );
    }

    #[test]
    fn serialize_with_custom_modules_identifier() {
        let rule: Box<dyn Rule> = Box::new(new_rule().with_modules_identifier("_CUSTOM_VAR"));

        assert_json_snapshot!("custom_modules_identifier", rule);
    }
}
