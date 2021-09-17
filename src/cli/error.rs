use darklua_core::ParserError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum CliError {
    Parser(PathBuf, ParserError),
    InputFile(String),
    InputFileNotFound(PathBuf),
    OutputFile(PathBuf, String),
    ConfigFileNotFound(PathBuf),
    ConfigFileReading(PathBuf),
    ConfigFileFormat(PathBuf, String),
    RuleError {
        file: PathBuf,
        rule_name: String,
        rule_number: usize,
        errors: Vec<String>,
    },
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Parser(path, error) => {
                write!(
                    f,
                    "could not parse input at <{}>: {}",
                    path.display(),
                    error,
                )
            }
            Self::InputFile(error) => {
                write!(f, "error while reading input file: {}", error)
            }
            Self::InputFileNotFound(path) => {
                write!(f, "input file not found: {}", path.display())
            }
            Self::OutputFile(path, error) => {
                write!(f, "error with output file <{}>: {}", path.display(), error)
            }
            Self::ConfigFileNotFound(path) => {
                write!(f, "can't find configuration file: {}", path.display())
            }
            Self::ConfigFileReading(path) => {
                write!(
                    f,
                    "error while reading configuration file: {}",
                    path.display()
                )
            }
            Self::ConfigFileFormat(path, error) => {
                write!(
                    f,
                    "format error in configuration file ({}): {}",
                    path.display(),
                    error
                )
            }
            Self::RuleError {
                file,
                rule_name,
                rule_number,
                errors,
            } => {
                write!(
                    f,
                    "error processing file `{}` [#{}] ({}):\n    * {}",
                    rule_name,
                    rule_number,
                    file.display(),
                    errors.join("\n    * "),
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_display_snapshot;

    #[test]
    fn snapshot_rule_error_with_only_one_error() {
        assert_display_snapshot!(
            "rule error with one error",
            CliError::RuleError {
                file: PathBuf::from("src/foo.lua"),
                rule_name: "the-rule-name".to_owned(),
                rule_number: 1,
                errors: vec!["some rule error happened!".to_owned()],
            }
        );
    }

    #[test]
    fn snapshot_rule_error_with_multiple_errors() {
        assert_display_snapshot!(
            "rule error with 3 errors",
            CliError::RuleError {
                file: PathBuf::from("src/foo.lua"),
                rule_name: "the-rule-name".to_owned(),
                rule_number: 1,
                errors: vec![
                    "the first error happened".to_owned(),
                    "then another one was registered".to_owned(),
                    "and finally it ended with this one".to_owned()
                ],
            }
        );
    }
}
