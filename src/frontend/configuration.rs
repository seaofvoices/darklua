use std::{fmt, marker::PhantomData, str::FromStr};

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{
    generator::{DenseLuaGenerator, LuaGenerator, ReadableLuaGenerator, TokenBasedLuaGenerator},
    nodes::Block,
    rules::{get_default_rules, Rule},
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
}

impl Configuration {
    #[inline]
    pub fn rules<'a, 'b: 'a>(&'b self) -> impl Iterator<Item = &'a dyn Rule> {
        self.rules.iter().map(AsRef::as_ref)
    }

    #[inline]
    pub fn build_parser(&self) -> Parser {
        self.generator.build_parser()
    }

    #[inline]
    pub fn generate_lua(&self, block: &Block, code: &str) -> String {
        self.generator.generate_lua(block, code)
    }

    #[inline]
    pub fn rules_len(&self) -> usize {
        self.rules.len()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            rules: get_default_rules(),
            generator: Default::default(),
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

    mod generator_parameters {
        use super::*;
        #[test]
        fn deserialize_retain_lines_params() {
            let config: Configuration =
                json5::from_str("{ generator: { name: 'retain-lines' } }").unwrap();

            assert_eq!(config.generator, GeneratorParameters::RetainLines);
        }

        #[test]
        fn deserialize_dense_params() {
            let config: Configuration = json5::from_str("{ generator: { name: 'dense' }}").unwrap();

            assert_eq!(
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

            assert_eq!(
                config.generator,
                GeneratorParameters::Dense { column_span: 110 }
            );
        }

        #[test]
        fn deserialize_readable_params() {
            let config: Configuration =
                json5::from_str("{ generator: { name: 'readable' } }").unwrap();

            assert_eq!(
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

            assert_eq!(
                config.generator,
                GeneratorParameters::Readable { column_span: 110 }
            );
        }

        #[test]
        fn deserialize_retain_lines_params_as_string() {
            let config: Configuration = json5::from_str("{generator: 'retain-lines'}").unwrap();

            assert_eq!(config.generator, GeneratorParameters::RetainLines);
        }

        #[test]
        fn deserialize_dense_params_as_string() {
            let config: Configuration = json5::from_str("{generator: 'dense'}").unwrap();

            assert_eq!(
                config.generator,
                GeneratorParameters::Dense {
                    column_span: DEFAULT_COLUMN_SPAN
                }
            );
        }

        #[test]
        fn deserialize_readable_params_as_string() {
            let config: Configuration = json5::from_str("{generator: 'readable'}").unwrap();

            assert_eq!(
                config.generator,
                GeneratorParameters::Readable {
                    column_span: DEFAULT_COLUMN_SPAN
                }
            );
        }
    }
}
