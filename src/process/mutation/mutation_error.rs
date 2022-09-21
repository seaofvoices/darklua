use std::{borrow::Cow, fmt};

use crate::process::path::{Component, NodePathBuf};

use super::Mutation;

#[derive(Clone, Debug, PartialEq, Eq)]
enum MutationErrorKind {
    UnexpectedPath { path: NodePathBuf },
    BlockPathExpected { path: NodePathBuf },
    StatementPathExpected { path: NodePathBuf },
    InvalidInsertion,
    Unspecified,
}

fn one_component_to_string(component: &Component) -> &'static str {
    match component {
        Component::Block(_) => "a block",
        Component::Expression(_) => "an expression",
        Component::Statement(_) => "a statement",
    }
}

fn format_expected_path(
    f: &mut fmt::Formatter<'_>,
    expected_component: &'static str,
    path: &NodePathBuf,
) -> fmt::Result {
    if let Some(last) = path.last() {
        write!(
            f,
            "{} path expected, but received path ending with {} `{}`",
            expected_component,
            one_component_to_string(last),
            path
        )
    } else {
        write!(
            f,
            "{} path expected, but received root path `{}`",
            expected_component, path,
        )
    }
}

impl fmt::Display for MutationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MutationErrorKind::UnexpectedPath { path } => {
                write!(f, "unexpected path `{}`", path)
            }
            MutationErrorKind::BlockPathExpected { path } => format_expected_path(f, "block", path),
            MutationErrorKind::StatementPathExpected { path } => {
                format_expected_path(f, "statement", path)
            }
            MutationErrorKind::InvalidInsertion => write!(f, "invalid insertion"),
            MutationErrorKind::Unspecified => write!(f, "unspecified"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutationError {
    mutation: Option<Mutation>,
    kind: MutationErrorKind,
    context: Option<Cow<'static, str>>,
}

impl MutationError {
    pub fn new(mutation: impl Into<Mutation>) -> Self {
        Self {
            mutation: Some(mutation.into()),
            kind: MutationErrorKind::Unspecified,
            context: None,
        }
    }

    pub fn mutation(&self) -> Option<&Mutation> {
        self.mutation.as_ref()
    }

    #[inline]
    fn kind(mut self, kind: MutationErrorKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn unexpected_path(self, path: impl Into<NodePathBuf>) -> Self {
        self.kind(MutationErrorKind::UnexpectedPath { path: path.into() })
    }

    pub fn block_path_expected(self, path: impl Into<NodePathBuf>) -> Self {
        self.kind(MutationErrorKind::BlockPathExpected { path: path.into() })
    }

    pub fn statement_path_expected(self, path: impl Into<NodePathBuf>) -> Self {
        self.kind(MutationErrorKind::StatementPathExpected { path: path.into() })
    }

    pub fn invalid_insertion(self) -> Self {
        self.kind(MutationErrorKind::InvalidInsertion)
    }

    pub fn context(mut self, context: impl Into<Cow<'static, str>>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_mutation(mut self, mutation: impl Into<Mutation>) -> Self {
        self.mutation = Some(mutation.into());
        self
    }
}

impl Default for MutationError {
    fn default() -> Self {
        Self {
            mutation: None,
            kind: MutationErrorKind::Unspecified,
            context: None,
        }
    }
}

impl fmt::Display for MutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(context) = self.context.as_ref().filter(|content| !content.is_empty()) {
            if self.kind == MutationErrorKind::Unspecified {
                write!(f, "{}", context)
            } else {
                write!(f, "{} ({})", self.kind, context)
            }
        } else {
            write!(f, "{}", self.kind)
        }
    }
}
