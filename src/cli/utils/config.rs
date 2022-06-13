use crate::cli::error::CliError;
use crate::cli::utils::DEFAULT_COLUMN_SPAN;

use darklua_core::generator::{
    DenseLuaGenerator, LuaGenerator, ReadableLuaGenerator, TokenBasedLuaGenerator,
};
use darklua_core::nodes::Block;
use darklua_core::rules::{get_default_rules, Rule};
use darklua_core::Parser;

use json5::from_str;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt, fs};

fn get_default_column_span() -> usize {
    DEFAULT_COLUMN_SPAN
}

const DEFAULT_CONFIG_PATHS: [&str; 2] = [".darklua.json", ".darklua.json5"];

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(default, deserialize_with = "string_or_struct")]
    generator: GeneratorParameters,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_span: Option<usize>,
    #[serde(alias = "process", default = "get_default_rules")]
    pub rules: Vec<Box<dyn Rule>>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("path", &self.path)
            .field("generator", &self.generator)
            .field("column_span", &self.column_span)
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

impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            generator: Default::default(),
            column_span: None,
            rules: get_default_rules(),
        }
    }
}

fn parse_string(content: &str, path: &Path) -> Result<Config, CliError> {
    from_str(content)
        .map_err(|error| CliError::ConfigFileFormat(path.to_owned(), error.to_string()))
}

impl Config {
    pub fn new(config_path: &Option<PathBuf>) -> Result<Self, CliError> {
        let config = if let Some(config_path) = config_path {
            if config_path.exists() {
                Self::read_file(config_path).map_err(CliError::from)?
            } else {
                return Err(CliError::ConfigFileNotFound(config_path.to_owned()));
            }
        } else {
            Self::read_default_file().map_err(CliError::from)?
        };

        if let Some(path) = &config.path {
            log::info!("Using configuration file: {}", path.to_string_lossy());
        } else {
            log::info!("Using default configuration");
        }

        Ok(config)
    }

    fn read_file(path: &Path) -> Result<Self, CliError> {
        fs::read_to_string(path)
            .map_err(|_| CliError::ConfigFileReading(path.to_owned()))
            .and_then(|content| parse_string(&content, path))
            .map(|config| config.with_path(path.to_owned()))
    }

    fn read_default_file() -> Result<Self, CliError> {
        DEFAULT_CONFIG_PATHS
            .iter()
            .map(Path::new)
            .filter(|path| path.is_file())
            .map(Self::read_file)
            .next()
            .unwrap_or_else(|| Ok(Self::default()))
    }

    fn with_path(mut self, path: PathBuf) -> Self {
        self.path.replace(path);
        self
    }

    pub fn generate_lua(&self, block: &Block, code: &str) -> String {
        self.generator.generate_lua(block, code)
    }

    pub fn build_parser(&self) -> Parser {
        self.generator.build_parser()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", tag = "name")]
enum GeneratorParameters {
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

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = String>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
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
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_retain_lines_params() {
        let config: Config = json5::from_str("{ generator: { name: 'retain-lines' } }").unwrap();

        assert_eq!(config.generator, GeneratorParameters::RetainLines);
    }

    #[test]
    fn deserialize_dense_params() {
        let config: Config = json5::from_str("{ generator: { name: 'dense' }}").unwrap();

        assert_eq!(
            config.generator,
            GeneratorParameters::Dense {
                column_span: DEFAULT_COLUMN_SPAN
            }
        );
    }

    #[test]
    fn deserialize_dense_params_with_column_span() {
        let config: Config =
            json5::from_str("{ generator: { name: 'dense', column_span: 110 } }").unwrap();

        assert_eq!(
            config.generator,
            GeneratorParameters::Dense { column_span: 110 }
        );
    }

    #[test]
    fn deserialize_readable_params() {
        let config: Config = json5::from_str("{ generator: { name: 'readable' } }").unwrap();

        assert_eq!(
            config.generator,
            GeneratorParameters::Readable {
                column_span: DEFAULT_COLUMN_SPAN
            }
        );
    }

    #[test]
    fn deserialize_readable_params_with_column_span() {
        let config: Config =
            json5::from_str("{ generator: { name: 'readable', column_span: 110 }}").unwrap();

        assert_eq!(
            config.generator,
            GeneratorParameters::Readable { column_span: 110 }
        );
    }

    #[test]
    fn deserialize_retain_lines_params_as_string() {
        let config: Config = json5::from_str("{generator: 'retain-lines'}").unwrap();

        assert_eq!(config.generator, GeneratorParameters::RetainLines);
    }

    #[test]
    fn deserialize_dense_params_as_string() {
        let config: Config = json5::from_str("{generator: 'dense'}").unwrap();

        assert_eq!(
            config.generator,
            GeneratorParameters::Dense {
                column_span: DEFAULT_COLUMN_SPAN
            }
        );
    }

    #[test]
    fn deserialize_readable_params_as_string() {
        let config: Config = json5::from_str("{generator: 'readable'}").unwrap();

        assert_eq!(
            config.generator,
            GeneratorParameters::Readable {
                column_span: DEFAULT_COLUMN_SPAN
            }
        );
    }
}
