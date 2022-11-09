use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::HashSet,
    fmt::{self, Display},
    path::PathBuf,
};

use crate::{rules::Rule, ParserError};

use super::{
    resources::ResourceError,
    work_item::{WorkData, WorkItem, WorkStatus},
};

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
    CyclicWork {
        work: Vec<(WorkData, Vec<PathBuf>)>,
    },
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

    pub(crate) fn cyclic_work(work_left: Vec<WorkItem>) -> Self {
        let source_left: HashSet<PathBuf> = work_left
            .iter()
            .map(|work| work.source().to_path_buf())
            .collect();

        let mut required_work: Vec<_> = work_left
            .into_iter()
            .filter_map(|work| {
                if work.total_required_content() == 0 {
                    None
                } else {
                    let (status, data) = work.extract();

                    match status {
                        WorkStatus::NotStarted => None,
                        WorkStatus::InProgress(progress) => {
                            let mut content: Vec<_> = progress
                                .required_content()
                                .filter(|path| source_left.contains(*path))
                                .map(PathBuf::from)
                                .collect();
                            if content.is_empty() {
                                None
                            } else {
                                content.sort();
                                Some((data, content))
                            }
                        }
                    }
                }
            })
            .collect();

        required_work.sort_by(|(a_data, a_content), (b_data, b_content)| {
            match a_content.len().cmp(&b_content.len()) {
                Ordering::Equal => a_data.source().cmp(b_data.source()),
                other => other,
            }
        });

        required_work.sort_by_key(|(_, content)| content.len());

        Self::new(ErrorKind::CyclicWork {
            work: required_work,
        })
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
            ErrorKind::CyclicWork { work } => {
                const MAX_PRINTED_WORK: usize = 12;
                const MAX_REQUIRED_PATH: usize = 20;

                let total = work.len();
                let list: Vec<_> = work
                    .iter()
                    .take(MAX_PRINTED_WORK)
                    .map(|(data, required)| {
                        let required_list: Vec<_> = required
                            .iter()
                            .take(MAX_REQUIRED_PATH)
                            .map(|path| format!("      - {}", path.display()))
                            .collect();

                        format!(
                            "    `{}` needs:\n{}",
                            data.source().display(),
                            required_list.join("\n")
                        )
                    })
                    .collect();

                write!(
                    f,
                    "cyclic work detected:\n{}{}",
                    list.join("\n"),
                    if total <= MAX_PRINTED_WORK {
                        "".to_owned()
                    } else {
                        format!("\n    and {} more", total - MAX_PRINTED_WORK)
                    }
                )?;
            }
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
