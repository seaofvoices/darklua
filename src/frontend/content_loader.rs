use std::{ffi::OsStr, io::Write, path::Path};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use crate::{
    nodes::{
        Block, Expression, FieldExpression, FunctionCall, Prefix, ReturnStatement,
        StringExpression, TableEntry, TableExpression,
    },
    process::to_expression,
    utils::Timer,
    DarkluaError, Parser, Resources,
};

/// Specifies how a file should be loaded and converted when processing or bundling.
///
/// The default loader for a file is inferred from its extension. Custom loaders
/// can be assigned to files via glob patterns in the configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Loader {
    /// Copies the file to the output.
    Copy,
    /// Parses and processes the file as Lua/Luau source code.
    Luau,
    /// Converts the file into a Lua module that returns the content as a string
    /// (`return "..."`).
    String,
    /// Converts the file into a Lua module that returns the content as a base64-encoded
    /// string (`return "..."`).
    #[serde(rename = "string/base64")]
    StringBase64,
    /// Converts the file into a Lua module that returns the content as a zstd-encoded
    #[serde(rename = "string/zstd")]
    StringZstd,
    /// Converts the file into a Lua module that returns the content as a gzip-encoded
    #[serde(rename = "string/gzip")]
    StringGzip,
    /// Converts the file into a Lua module that returns the content as a zlib-encoded
    #[serde(rename = "string/zlib")]
    StringZlib,
    /// Converts the file into a Lua module that returns the content as a buffer
    /// (`return buffer.fromstring("...")`).
    Buffer,
    /// Converts the file into a Lua module that returns the content as a base64-encoded
    /// buffer (`return buffer.fromstring("...")`).
    #[serde(rename = "buffer/base64")]
    BufferBase64,
    /// Converts the file into a Lua module that returns the content as a zstd-encoded
    #[serde(rename = "buffer/zstd")]
    BufferZstd,
    /// Converts the file into a Lua module that returns the content as a gzip-encoded
    #[serde(rename = "buffer/gzip")]
    BufferGzip,
    /// Converts the file into a Lua module that returns the content as a zlib-encoded
    #[serde(rename = "buffer/zlib")]
    BufferZlib,
    /// Converts the file into a Lua module that returns the content as a byte array
    /// (`return { ... }`).
    Bytes,
    /// Converts the file into a Lua module that returns the content as a base64-encoded
    /// byte array (`return { ... }`).
    #[serde(rename = "bytes/base64")]
    BytesBase64,
    /// Converts the file into a Lua module that returns the content as a zstd-encoded
    #[serde(rename = "bytes/zstd")]
    BytesZstd,
    /// Converts the file into a Lua module that returns the content as a gzip-encoded
    #[serde(rename = "bytes/gzip")]
    BytesGzip,
    /// Converts the file into a Lua module that returns the content as a zlib-encoded
    #[serde(rename = "bytes/zlib")]
    BytesZlib,
    /// Converts the file into a Lua module that returns the JSON data.
    Json,
    /// Converts the file into a Lua module that returns the JSON-lines data.
    JsonLines,
    /// Converts the file into a Lua module that returns the TOML data.
    Toml,
    /// Converts the file into a Lua module that returns the YAML data.
    #[serde(alias = "yml")]
    Yaml,
    /// Ignores the file.
    Skip,
}

impl Loader {
    pub(crate) fn from_extension(extension: &OsStr) -> Option<Self> {
        let extension = extension.to_str()?;

        match extension {
            "luau" | "lua" => Some(Self::Luau),
            "json" | "json5" => Some(Self::Json),
            "jsonl" | "ndjson" => Some(Self::JsonLines),
            "toml" => Some(Self::Toml),
            "yaml" | "yml" => Some(Self::Yaml),
            "txt" => Some(Self::String),
            _ => None,
        }
    }

    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        path.extension().and_then(Self::from_extension)
    }

    pub(crate) fn to_internal_loader(self) -> InternalLoader {
        match self {
            Self::Copy => InternalLoader::Copy,
            Self::Luau => InternalLoader::Luau,
            Self::String => InternalLoader::String(LoaderEncoding::None),
            Self::StringBase64 => InternalLoader::String(LoaderEncoding::Base64),
            Self::StringZstd => InternalLoader::String(LoaderEncoding::Zstd),
            Self::StringGzip => InternalLoader::String(LoaderEncoding::Gzip),
            Self::StringZlib => InternalLoader::String(LoaderEncoding::Zlib),
            Self::Buffer => InternalLoader::Buffer(LoaderEncoding::None),
            Self::BufferBase64 => InternalLoader::Buffer(LoaderEncoding::Base64),
            Self::BufferZstd => InternalLoader::Buffer(LoaderEncoding::Zstd),
            Self::BufferGzip => InternalLoader::Buffer(LoaderEncoding::Gzip),
            Self::BufferZlib => InternalLoader::Buffer(LoaderEncoding::Zlib),
            Self::Bytes => InternalLoader::Bytes(LoaderEncoding::None),
            Self::BytesBase64 => InternalLoader::Bytes(LoaderEncoding::Base64),
            Self::BytesZstd => InternalLoader::Bytes(LoaderEncoding::Zstd),
            Self::BytesGzip => InternalLoader::Bytes(LoaderEncoding::Gzip),
            Self::BytesZlib => InternalLoader::Bytes(LoaderEncoding::Zlib),
            Self::Json => InternalLoader::Json,
            Self::JsonLines => InternalLoader::JsonLines,
            Self::Toml => InternalLoader::Toml,
            Self::Yaml => InternalLoader::Yaml,
            Self::Skip => InternalLoader::Skip,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InternalLoader {
    Copy,
    Luau,
    String(LoaderEncoding),
    Buffer(LoaderEncoding),
    Bytes(LoaderEncoding),
    Json,
    JsonLines,
    Toml,
    Yaml,
    Skip,
}

impl InternalLoader {
    pub(crate) fn outputs_lua(&self) -> bool {
        match self {
            Self::Luau
            | Self::String(_)
            | Self::Buffer(_)
            | Self::Bytes(_)
            | Self::Json
            | Self::JsonLines
            | Self::Toml
            | Self::Yaml => true,
            Self::Copy | Self::Skip => false,
        }
    }

    pub(crate) fn load(
        &self,
        source: &Path,
        resources: &Resources,
        parser: &Parser,
    ) -> Result<ContentType, DarkluaError> {
        match self {
            Self::Copy => {
                let content = resources.get_bytes(source)?;
                Ok(ContentType::Copied(content))
            }
            Self::Skip => Ok(ContentType::None),
            Self::Luau => {
                let content = resources.get(source)?;
                let parser_timer = Timer::now();

                let block = parser
                    .parse(&content)
                    .map_err(|parser_error| DarkluaError::parser_error(source, parser_error))?;

                let parser_time = parser_timer.duration_label();
                log::debug!("parsed `{}` in {}", source.display(), parser_time);

                Ok(ContentType::Parsed {
                    block,
                    source: content,
                })
            }
            Self::String(encoding) => {
                let content = resources.get_bytes(source)?;

                let new_module = Block::default().with_last_statement(ReturnStatement::one(
                    StringExpression::from_value(encoding.encode(&content)?.unwrap_or(content)),
                ));

                Ok(ContentType::Block(new_module))
            }
            Self::Buffer(encoding) => {
                let content = resources.get_bytes(source)?;

                let new_module = Block::default().with_last_statement(ReturnStatement::one(
                    FunctionCall::from_prefix(FieldExpression::new(
                        Prefix::from_name("buffer"),
                        "fromstring",
                    ))
                    .with_arguments(StringExpression::from_value(
                        encoding.encode(&content)?.unwrap_or(content),
                    )),
                ));

                Ok(ContentType::Block(new_module))
            }
            Self::Bytes(encoding) => {
                let content = resources.get_bytes(source)?;

                let new_module = Block::default().with_last_statement(ReturnStatement::one(
                    TableExpression::new(
                        encoding
                            .encode(&content)?
                            .unwrap_or(content)
                            .iter()
                            .map(TableEntry::from_value)
                            .collect(),
                    ),
                ));

                Ok(ContentType::Block(new_module))
            }
            Self::Json => {
                let content = resources.get(source)?;
                let data =
                    json5::from_str::<serde_json::Value>(&content).map_err(DarkluaError::from)?;

                ContentType::from_data("json", data, source)
            }
            Self::JsonLines => {
                let content = resources.get(source)?;

                let mut data = Vec::new();
                for (index, line) in content.trim_end().lines().enumerate() {
                    let element = json5::from_str::<serde_json::Value>(line).map_err(|err| {
                        DarkluaError::from(err)
                            .context(format!("failed to parse JSON entry at line {}", index + 1))
                    })?;
                    data.push(element);
                }

                ContentType::from_data("json", serde_json::Value::Array(data), source)
            }
            Self::Toml => {
                let content = resources.get(source)?;
                let data = toml::from_str::<toml::Value>(&content).map_err(DarkluaError::from)?;

                ContentType::from_data("toml", data, source)
            }
            Self::Yaml => {
                let content = resources.get(source)?;
                let data = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    .map_err(DarkluaError::from)?;

                ContentType::from_data("yaml", data, source)
            }
        }
    }
}

pub(crate) enum ContentType {
    None,
    Copied(Vec<u8>),
    Expression(Expression),
    Block(Block),
    Parsed { block: Block, source: String },
}

impl ContentType {
    pub(crate) fn from_data(
        label: &'static str,
        value: impl Serialize,
        source: &Path,
    ) -> Result<Self, DarkluaError> {
        log::trace!(
            "transcode {} data to Lua from `{}`",
            label,
            source.display()
        );
        let transcode_duration = Timer::now();
        let expression = to_expression(&value).map_err(DarkluaError::from)?;
        log::debug!(
            "transcoded {} data to Lua from `{}` in {}",
            label,
            source.display(),
            transcode_duration.duration_label()
        );
        Ok(Self::Expression(expression))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoaderEncoding {
    None,
    Base64,
    Zstd,
    Gzip,
    Zlib,
}

impl LoaderEncoding {
    pub(crate) fn encode(&self, content: &[u8]) -> Result<Option<Vec<u8>>, DarkluaError> {
        match self {
            Self::None => Ok(None),
            Self::Base64 => encode_base64(content).map(Some),
            Self::Zstd => encode_zstd(content).map(Some),
            Self::Gzip => encode_gzip(content).map(Some),
            Self::Zlib => encode_zlib(content).map(Some),
        }
    }
}

fn encode_base64(content: &[u8]) -> Result<Vec<u8>, DarkluaError> {
    let mut encoded_content = vec![0; content.len() * 4 / 3 + 4];

    let bytes_written = STANDARD
        .encode_slice(content, &mut encoded_content)
        .map_err(|err| DarkluaError::custom(format!("failed to encode base64: {}", err)))?;

    encoded_content.truncate(bytes_written);

    Ok(encoded_content)
}

fn encode_zstd(content: &[u8]) -> Result<Vec<u8>, DarkluaError> {
    let compression_level = 7;

    zstd::stream::encode_all(content, compression_level)
        .map_err(|err| DarkluaError::custom(format!("failed to encode with zstd: {}", err)))
}

fn encode_gzip(content: &[u8]) -> Result<Vec<u8>, DarkluaError> {
    use flate2::write::GzEncoder;

    let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(content).map_err(|err| {
        DarkluaError::custom(format!("failed to write content to gzip encoder: {}", err))
    })?;

    encoder
        .finish()
        .map_err(|err| DarkluaError::custom(format!("failed to encode with gzip: {}", err)))
}

fn encode_zlib(content: &[u8]) -> Result<Vec<u8>, DarkluaError> {
    use flate2::write::ZlibEncoder;

    let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(content).map_err(|err| {
        DarkluaError::custom(format!("failed to write content to zlib encoder: {}", err))
    })?;

    encoder
        .finish()
        .map_err(|err| DarkluaError::custom(format!("failed to encode with zlib: {}", err)))
}
