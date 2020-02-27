use darklua_core::ParsingError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

pub enum CliError {
    Parser(PathBuf, ParsingError),
    InputFile(String),
    InputFileNotFound(PathBuf),
    OutputFile(PathBuf, String),
    ConfigFileNotFound(PathBuf),
    ConfigFileReading(PathBuf),
    ConfigFileFormat(PathBuf, String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Parser(path, error) => {
                write!(f, "could not parse input at <{}>: {:?}", path.to_string_lossy(), error)
            }
            Self::InputFile(error) => {
                write!(f, "error while reading input file: {}", error)
            }
            Self::InputFileNotFound(path) => {
                write!(f, "input file not found: {}", path.to_string_lossy())
            }
            Self::OutputFile(path, error) => {
                write!(f, "error with output file <{}>: {}", path.to_string_lossy(), error)
            }
            Self::ConfigFileNotFound(path) => {
                write!(f, "can't find configuration file: {}", path.to_string_lossy())
            }
            Self::ConfigFileReading(path) => {
                write!(f, "error while reading configuration file: {}", path.to_string_lossy())
            }
            Self::ConfigFileFormat(path, error) => {
                write!(f, "format error in configuration file ({}): {}", path.to_string_lossy(), error)
            }
        }
    }
}
