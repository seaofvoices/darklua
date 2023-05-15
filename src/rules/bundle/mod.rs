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

pub const BUNDLER_RULE_NAME: &str = "bundler";

/// A rule that inlines required modules
#[derive(Debug)]
pub(crate) struct Bundler {
    parser: Parser,
    extra_module_relative_location: Option<PathBuf>,
    modules_identifier: String,
    require_mode: RequireMode,
}

impl Bundler {
    pub(crate) fn with_modules_identifier(mut self, modules_identifier: impl Into<String>) -> Self {
        self.modules_identifier = modules_identifier.into();
        self
    }

    pub(crate) fn with_require_mode(mut self, require_mode: impl Into<RequireMode>) -> Self {
        self.require_mode = require_mode.into();
        self
    }

    pub(crate) fn with_parser(mut self, parser: Parser) -> Self {
        self.parser = parser;
        self
    }

    pub(crate) fn with_configuration_location(mut self, location: impl AsRef<Path>) -> Self {
        let location = location
            .as_ref()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        self.extra_module_relative_location = Some(location);
        self
    }
}

impl Rule for Bundler {
    fn process(&self, block: &mut Block, context: &Context) -> RuleProcessResult {
        self.require_mode.process_block(
            block,
            context.current_path().to_path_buf(),
            self.extra_module_relative_location.as_deref(),
            &self.modules_identifier,
            context.resources(),
            &self.parser,
        )
    }
}

impl RuleConfiguration for Bundler {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_required_properties(&properties, &["require-mode"])?;

        for (key, value) in properties {
            match key.as_str() {
                "modules-identifier" => match value {
                    RulePropertyValue::String(identifier) => {
                        self.modules_identifier = identifier;
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

        if self.modules_identifier != DEFAULT_MODULE_IDENTIFIER {
            properties.insert(
                "modules-identifier".to_owned(),
                RulePropertyValue::from(&self.modules_identifier),
            );
        }

        properties
    }
}

const DEFAULT_MODULE_IDENTIFIER: &str = "__DARKLUA_BUNDLE_MODULES";

impl Default for Bundler {
    fn default() -> Self {
        Self {
            modules_identifier: DEFAULT_MODULE_IDENTIFIER.to_owned(),
            extra_module_relative_location: None,
            require_mode: RequireMode::default(),
            parser: Parser::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    fn new_rule() -> Bundler {
        Bundler::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_bundler", rule);
    }

    #[test]
    fn serialize_path_require_mode_with_custom_module_folder_name() {
        let rule: Box<dyn Rule> =
            Box::new(new_rule().with_require_mode(PathRequireMode::new("__init__")));

        assert_json_snapshot!("path_require_mode_with_custom_module_folder_name", rule);
    }

    #[test]
    fn serialize_path_require_mode_with_custom_module_folder_name_and_modules_identifier() {
        let rule: Box<dyn Rule> = Box::new(
            new_rule()
                .with_require_mode(PathRequireMode::new("__init__"))
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
