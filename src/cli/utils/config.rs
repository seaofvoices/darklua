use crate::cli::GlobalOptions;
use crate::cli::error::CliError;
use crate::cli::utils::DEFAULT_COLUMN_SPAN;

use darklua_core::rules::{get_default_rules, Rule};

use json5::from_str;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};

fn get_default_column_span() -> usize { DEFAULT_COLUMN_SPAN }

const DEFAULT_CONFIG_PATHS: [&str; 2] = [".darklua.json", ".darklua.json5"];

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(default = "get_default_column_span")]
    pub column_span: usize,
    #[serde(default = "get_default_rules")]
    pub process: Vec<Box<dyn Rule>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            column_span: get_default_column_span(),
            process: get_default_rules(),
        }
    }
}

fn parse_string(content: &str, path: &Path) -> Result<Config, CliError> {
    from_str(content)
        .map_err(|error| CliError::ConfigFileFormat(path.to_owned(), error.to_string()))
}

impl Config {
    pub fn new(config_path: &Option<PathBuf>, global: &GlobalOptions) -> Result<Self, CliError> {
        let config = if let Some(config_path) = config_path {
            if config_path.exists() {
                Self::read_file(config_path).map_err(CliError::from)?
            } else {
                return Err(CliError::ConfigFileNotFound(config_path.to_owned()))
            }
        } else {
            Self::read_default_file().map_err(CliError::from)?
        };

        if global.verbose > 0 {
            if let Some(path) = &config.path {
                println!("Using configuration file: {}", path.to_string_lossy());
            } else {
                println!("Using default configuration");
            }
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
        DEFAULT_CONFIG_PATHS.iter()
            .map(|path| Path::new(path))
            .filter(|path| path.exists())
            .find_map(|path| Some(Self::read_file(path)))
            .unwrap_or_else(|| Ok(Self::default()))
    }

    fn with_path(mut self, path: PathBuf) -> Self {
        self.path.replace(path);
        self
    }
}

