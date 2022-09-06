use super::StatementSpan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MutationEffect {
    StatementRemoved(StatementSpan),
    StatementAdded(StatementSpan),
}

impl MutationEffect {
    pub fn statement_removed(span: impl Into<StatementSpan>) -> Self {
        Self::StatementRemoved(span.into())
    }

    pub fn statement_added(path: impl Into<StatementSpan>) -> Self {
        Self::StatementAdded(path.into())
    }
}
