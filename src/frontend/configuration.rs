use std::{
    fmt,
    marker::PhantomData,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{de, Deserialize, Deserializer, Serialize};

use crate::{
    generator::{DenseLuaGenerator, LuaGenerator, ReadableLuaGenerator, TokenBasedLuaGenerator},
    nodes::Block,
    rules::{bundle::RequireMode, get_default_rules, Rule},
    Parser,
};

const DEFAULT_COLUMN_SPAN: usize = 80;

fn get_default_column_span() -> usize {
    DEFAULT_COLUMN_SPAN
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(alias = "process", default = "get_default_rules")]
    rules: Vec<Box<dyn Rule>>,
    #[serde(default, deserialize_with = "string_or_struct")]
    generator: GeneratorParameters,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bundle: Option<BundleConfiguration>,
    #[serde(default, skip)]
    location: Option<PathBuf>,
}

impl Configuration {
    /// Creates a configuration object without any rules and with the default
    /// generator
    pub fn empty() -> Self {
        Self {
            rules: Vec::new(),
            generator: GeneratorParameters::default(),
            bundle: None,
            location: None,
        }
    }

    #[inline]
    pub fn with_generator(mut self, generator: GeneratorParameters) -> Self {
        self.generator = generator;
        self
    }

    #[inline]
    pub fn with_rule(mut self, rule: impl Into<Box<dyn Rule>>) -> Self {
        self.push_rule(rule);
        self
    }

    #[inline]
    pub fn with_bundle_configuration(mut self, configuration: BundleConfiguration) -> Self {
        self.bundle = Some(configuration);
        self
    }

    #[inline]
    pub fn with_location(mut self, location: impl Into<PathBuf>) -> Self {
        self.location = Some(location.into());
        self
    }

    #[inline]
    pub fn push_rule(&mut self, rule: impl Into<Box<dyn Rule>>) {
        self.rules.push(rule.into());
    }

    #[inline]
    pub(crate) fn rules<'a, 'b: 'a>(&'b self) -> impl Iterator<Item = &'a dyn Rule> {
        self.rules.iter().map(AsRef::as_ref)
    }

    #[inline]
    pub(crate) fn build_parser(&self) -> Parser {
        self.generator.build_parser()
    }

    #[inline]
    pub(crate) fn generate_lua(&self, block: &Block, code: &str) -> String {
        self.generator.generate_lua(block, code)
    }

    #[inline]
    pub(crate) fn bundle(&self) -> Option<&BundleConfiguration> {
        self.bundle.as_ref()
    }

    #[inline]
    pub(crate) fn location(&self) -> Option<&Path> {
        self.location.as_ref().map(AsRef::as_ref)
    }

    #[inline]
    pub(crate) fn rules_len(&self) -> usize {
        self.rules.len()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            rules: get_default_rules(),
            generator: Default::default(),
            bundle: None,
            location: None,
        }
    }
}

impl std::fmt::Debug for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("generator", &self.generator)
            .field(
                "rules",
                &self
                    .rules
                    .iter()
                    .map(|rule| {
                        json5::to_string(rule)
                            .ok()
                            .unwrap_or_else(|| rule.get_name().to_owned())
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            )
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "name")]
pub enum GeneratorParameters {
    RetainLines,
    Dense {
        #[serde(default = "get_default_column_span")]
        column_span: usize,
    },
    Readable {
        #[serde(default = "get_default_column_span")]
        column_span: usize,
    },
}

impl Default for GeneratorParameters {
    fn default() -> Self {
        Self::RetainLines
    }
}

impl GeneratorParameters {
    pub fn default_dense() -> Self {
        Self::Dense {
            column_span: DEFAULT_COLUMN_SPAN,
        }
    }

    pub fn default_readable() -> Self {
        Self::Readable {
            column_span: DEFAULT_COLUMN_SPAN,
        }
    }

    fn generate_lua(&self, block: &Block, code: &str) -> String {
        match self {
            Self::RetainLines => {
                let mut generator = TokenBasedLuaGenerator::new(code);
                generator.write_block(block);
                generator.into_string()
            }
            Self::Dense { column_span } => {
                let mut generator = DenseLuaGenerator::new(*column_span);
                generator.write_block(block);
                generator.into_string()
            }
            Self::Readable { column_span } => {
                let mut generator = ReadableLuaGenerator::new(*column_span);
                generator.write_block(block);
                generator.into_string()
            }
        }
    }

    fn build_parser(&self) -> Parser {
        match self {
            Self::RetainLines => Parser::default().preserve_tokens(),
            Self::Dense { .. } | Self::Readable { .. } => Parser::default(),
        }
    }
}

impl FromStr for GeneratorParameters {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "retain-lines" => Self::RetainLines,
            "dense" => Self::Dense {
                column_span: DEFAULT_COLUMN_SPAN,
            },
            "readable" => Self::Readable {
                column_span: DEFAULT_COLUMN_SPAN,
            },
            _ => return Err(format!("invalid generator name `{}`", s)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct BundleConfiguration {
    #[serde(deserialize_with = "string_or_struct")]
    require_mode: RequireMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    modules_identifier: Option<String>,
}

impl BundleConfiguration {
    pub(crate) fn require_mode(&self) -> &RequireMode {
        &self.require_mode
    }

    pub(crate) fn modules_identifier(&self) -> Option<&str> {
        self.modules_identifier.as_ref().map(AsRef::as_ref)
    }
}

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = String>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<T>);

    impl<'de, T> de::Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = String>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or object")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            T::from_str(value).map_err(E::custom)
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

#[cfg(test)]
mod test {
    use super::*;

    mod generator_parameters {
        use super::*;
        #[test]
        fn deserialize_retain_lines_params() {
            let config: Configuration =
                json5::from_str("{ generator: { name: 'retain-lines' } }").unwrap();

            pretty_assertions::assert_eq!(config.generator, GeneratorParameters::RetainLines);
        }

        #[test]
        fn deserialize_dense_params() {
            let config: Configuration = json5::from_str("{ generator: { name: 'dense' }}").unwrap();

            pretty_assertions::assert_eq!(
                config.generator,
                GeneratorParameters::Dense {
                    column_span: DEFAULT_COLUMN_SPAN
                }
            );
        }

        #[test]
        fn deserialize_dense_params_with_column_span() {
            let config: Configuration =
                json5::from_str("{ generator: { name: 'dense', column_span: 110 } }").unwrap();

            pretty_assertions::assert_eq!(
                config.generator,
                GeneratorParameters::Dense { column_span: 110 }
            );
        }

        #[test]
        fn deserialize_readable_params() {
            let config: Configuration =
                json5::from_str("{ generator: { name: 'readable' } }").unwrap();

            pretty_assertions::assert_eq!(
                config.generator,
                GeneratorParameters::Readable {
                    column_span: DEFAULT_COLUMN_SPAN
                }
            );
        }

        #[test]
        fn deserialize_readable_params_with_column_span() {
            let config: Configuration =
                json5::from_str("{ generator: { name: 'readable', column_span: 110 }}").unwrap();

            pretty_assertions::assert_eq!(
                config.generator,
                GeneratorParameters::Readable { column_span: 110 }
            );
        }

        #[test]
        fn deserialize_retain_lines_params_as_string() {
            let config: Configuration = json5::from_str("{generator: 'retain-lines'}").unwrap();

            pretty_assertions::assert_eq!(config.generator, GeneratorParameters::RetainLines);
        }

        #[test]
        fn deserialize_dense_params_as_string() {
            let config: Configuration = json5::from_str("{generator: 'dense'}").unwrap();

            pretty_assertions::assert_eq!(
                config.generator,
                GeneratorParameters::Dense {
                    column_span: DEFAULT_COLUMN_SPAN
                }
            );
        }

        #[test]
        fn deserialize_readable_params_as_string() {
            let config: Configuration = json5::from_str("{generator: 'readable'}").unwrap();

            pretty_assertions::assert_eq!(
                config.generator,
                GeneratorParameters::Readable {
                    column_span: DEFAULT_COLUMN_SPAN
                }
            );
        }

        #[test]
        fn deserialize_unknown_generator_name() {
            let result: Result<Configuration, _> = json5::from_str("{generator: 'oops'}");

            pretty_assertions::assert_eq!(
                result.expect_err("deserialization should fail").to_string(),
                "invalid generator name `oops`"
            );
        }
    }

    mod bundle_configuration {
        use crate::rules::bundle::PathRequireMode;

        use super::*;

        #[test]
        fn deserialize_path_require_mode_as_string() {
            let config: Configuration =
                json5::from_str("{ bundle: { 'require-mode': 'path' } }").unwrap();

            pretty_assertions::assert_eq!(
                config.bundle.unwrap(),
                BundleConfiguration {
                    require_mode: RequireMode::Path(Default::default()),
                    modules_identifier: None
                }
            );
        }

        #[test]
        fn deserialize_path_require_mode_as_object() {
            let config: Configuration =
                json5::from_str("{bundle: { 'require-mode': { name: 'path' } } }").unwrap();

            pretty_assertions::assert_eq!(
                config.bundle.unwrap(),
                BundleConfiguration {
                    require_mode: RequireMode::Path(Default::default()),
                    modules_identifier: None
                }
            );
        }

        #[test]
        fn deserialize_path_require_mode_with_custom_module_folder_name() {
            let config: Configuration = json5::from_str(
                "{bundle: { 'require-mode': { name: 'path', 'module-folder-name': '__INIT__' } } }",
            )
            .unwrap();

            pretty_assertions::assert_eq!(
                config.bundle.unwrap(),
                BundleConfiguration {
                    require_mode: PathRequireMode::new("__INIT__").into(),
                    modules_identifier: None
                }
            );
        }

        #[test]
        fn deserialize_path_require_mode_with_custom_module_identifier() {
            let config: Configuration = json5::from_str(
                "{bundle: { 'require-mode': 'path', 'modules-identifier': '__M' } }",
            )
            .unwrap();

            pretty_assertions::assert_eq!(
                config.bundle.unwrap(),
                BundleConfiguration {
                    require_mode: RequireMode::Path(Default::default()),
                    modules_identifier: Some("__M".to_owned())
                }
            );
        }

        #[test]
        fn deserialize_path_require_mode_with_custom_module_identifier_and_module_folder_name() {
            let config: Configuration = json5::from_str(
                "{bundle: { 'require-mode': { name: 'path', 'module-folder-name': '__INIT__' }, 'modules-identifier': '__M' } }",
            )
            .unwrap();

            pretty_assertions::assert_eq!(
                config.bundle.unwrap(),
                BundleConfiguration {
                    require_mode: PathRequireMode::new("__INIT__").into(),
                    modules_identifier: Some("__M".to_owned())
                }
            );
        }

        #[test]
        fn deserialize_path_require_mode_with_excludes() {
            let config: Configuration = json5::from_str(
                "{bundle: { 'require-mode': { name: 'path', 'excludes': ['@lune', 'secrets'] } } }",
            )
            .unwrap();

            pretty_assertions::assert_eq!(
                config.bundle.unwrap(),
                BundleConfiguration {
                    require_mode: PathRequireMode::default()
                        .with_exclude("@lune")
                        .with_exclude("secrets")
                        .into(),
                    modules_identifier: None
                }
            );
        }

        #[test]
        fn deserialize_unknown_require_mode_name() {
            let result: Result<Configuration, _> =
                json5::from_str("{bundle: { 'require-mode': 'oops' } }");

            pretty_assertions::assert_eq!(
                result.expect_err("deserialization should fail").to_string(),
                "invalid require mode `oops`"
            );
        }
    }
}
