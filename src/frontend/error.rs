use std::{
    borrow::Cow,
    fmt::{self, Display},
    path::PathBuf,
};

use crate::{rules::Rule, ParserError};

use super::{resources::ResourceError, work_item::WorkItem};

#[derive(Debug, Clone)]
enum ErrorKind {
    Parser {
        path: PathBuf,
        error: ParserError,
    },
    ResourceNotFound {
        path: PathBuf,
    },
    InvalidConfiguration {
        path: PathBuf,
    },
    MultipleConfigurationFound {
        paths: Vec<PathBuf>,
    },
    IO {
        path: PathBuf,
        error: String,
    },
    UncachedWork {
        path: PathBuf,
    },
    RuleError {
        path: PathBuf,
        rule_name: String,
        rule_number: usize,
        error: String,
    },
    CyclicWork,
    Custom {
        message: Cow<'static, str>,
    },
}

pub type DarkluaResult<T> = Result<T, DarkluaError>;

#[derive(Debug, Clone)]
pub struct DarkluaError {
    kind: ErrorKind,
    context: Option<Cow<'static, str>>,
}

impl DarkluaError {
    fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            context: None,
        }
    }

    pub(crate) fn context(mut self, context: impl Into<Cow<'static, str>>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub(crate) fn parser_error(path: impl Into<PathBuf>, error: ParserError) -> Self {
        Self::new(ErrorKind::Parser {
            path: path.into(),
            error,
        })
    }

    pub(crate) fn multiple_configuration_found(
        configuration_files: impl Iterator<Item = PathBuf>,
    ) -> Self {
        Self::new(ErrorKind::MultipleConfigurationFound {
            paths: configuration_files.collect(),
        })
    }

    pub(crate) fn io_error(path: impl Into<PathBuf>, error: impl Into<String>) -> Self {
        Self::new(ErrorKind::IO {
            path: path.into(),
            error: error.into(),
        })
    }

    pub(crate) fn resource_not_found(path: impl Into<PathBuf>) -> Self {
        Self::new(ErrorKind::ResourceNotFound { path: path.into() })
    }

    pub(crate) fn invalid_configuration_file(path: impl Into<PathBuf>) -> Self {
        Self::new(ErrorKind::InvalidConfiguration { path: path.into() })
    }

    pub(crate) fn uncached_work(path: impl Into<PathBuf>) -> Self {
        Self::new(ErrorKind::UncachedWork { path: path.into() })
    }

    pub(crate) fn rule_error(
        path: impl Into<PathBuf>,
        rule: &dyn Rule,
        rule_index: usize,
        rule_error: impl Into<String>,
    ) -> Self {
        Self::new(ErrorKind::RuleError {
            path: path.into(),
            rule_name: rule.get_name().to_owned(),
            rule_number: rule_index,
            error: rule_error.into(),
        })
    }

    pub(crate) fn cyclic_work(mut work_left: Vec<WorkItem>) -> Self {
        work_left.retain(WorkItem::has_started);
        work_left.sort_by(|a, b| a.total_required_content().cmp(&b.total_required_content()));
        // todo: provide a nice error message from the work left
        Self::new(ErrorKind::CyclicWork)
    }

    pub(crate) fn custom(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrorKind::Custom {
            message: message.into(),
        })
    }
}

impl From<ResourceError> for DarkluaError {
    fn from(err: ResourceError) -> Self {
        match err {
            ResourceError::NotFound(path) => DarkluaError::resource_not_found(path),
            ResourceError::IO { path, error } => DarkluaError::io_error(path, error),
        }
    }
}

impl Display for DarkluaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Parser { path, error } => {
                write!(f, "unable to parse `{}`: {}", path.display(), error)?;
            }
            ErrorKind::ResourceNotFound { path } => {
                write!(f, "unable to find `{}`", path.display())?;
            }
            ErrorKind::InvalidConfiguration { path } => {
                write!(f, "invalid configuration file at `{}`", path.display())?;
            }
            ErrorKind::MultipleConfigurationFound { paths } => {
                write!(
                    f,
                    "multiple default configuration file found: {}",
                    paths
                        .iter()
                        .map(|path| format!("`{}`", path.display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }
            ErrorKind::IO { path, error } => {
                write!(f, "IO error with `{}`: {}", path.display(), error)?;
            }
            ErrorKind::UncachedWork { path } => {
                write!(f, "attempt to obtain work at `{}`", path.display())?;
            }
            ErrorKind::RuleError {
                path,
                rule_name,
                rule_number,
                error,
            } => {
                write!(
                    f,
                    "error processing `{}` ({} [#{}]): {}",
                    path.display(),
                    rule_name,
                    rule_number,
                    error,
                )?;
            }
            ErrorKind::CyclicWork => {
                todo!()
            },
            ErrorKind::Custom { message } => {
                write!(f, "{}", message)?;
            }
        };

        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }

        Ok(())
    }
}
