use std::borrow;

use crate::{
    nodes::Block,
    process::path::{NodePathBuf, NodePathSlice},
};

use super::{
    statement_insertion::StatementInsertion, statement_replacement::StatementReplacement,
    MutationEffect, MutationResult, StatementInsertionContent, StatementSpan,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum MutationKind {
    StatementInsertion(StatementInsertion),
    StatementReplacement(StatementReplacement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mutation {
    kind: MutationKind,
}

impl Mutation {
    #[inline]
    pub fn remove(statement_span: impl Into<StatementSpan>) -> Self {
        MutationKind::StatementReplacement(StatementReplacement::remove(statement_span)).into()
    }

    #[inline]
    pub fn replace(
        statement_span: impl Into<StatementSpan>,
        insertion: impl Into<StatementInsertionContent>,
    ) -> Self {
        let insertion = insertion.into();
        if insertion.is_empty() {
            Self::remove(statement_span)
        } else {
            MutationKind::StatementReplacement(StatementReplacement::replace(
                statement_span,
                insertion,
            ))
            .into()
        }
    }

    #[inline]
    pub fn insert_before(
        statement_path: impl Into<NodePathBuf>,
        insertion: impl Into<StatementInsertionContent>,
    ) -> Self {
        MutationKind::StatementInsertion(StatementInsertion::insert_before(
            statement_path,
            insertion,
        ))
        .into()
    }

    #[inline]
    pub fn insert_after(
        statement_path: impl borrow::Borrow<NodePathSlice>,
        insertion: impl Into<StatementInsertionContent>,
    ) -> Self {
        MutationKind::StatementInsertion(StatementInsertion::insert_after(
            statement_path,
            insertion,
        ))
        .into()
    }

    pub fn apply(self, block: &mut Block) -> MutationResult {
        match self.kind {
            MutationKind::StatementInsertion(mutation) => mutation.apply(block),
            MutationKind::StatementReplacement(mutation) => mutation.apply(block),
        }
    }

    /// Apply a given effect to the mutation and return true if the mutation
    /// should be kept or false if it should be discarded.
    pub fn mutate(&mut self, effect: &MutationEffect) -> bool {
        match &mut self.kind {
            MutationKind::StatementInsertion(mutation) => mutation.mutate(effect),
            MutationKind::StatementReplacement(mutation) => mutation.mutate(effect),
        }
    }
}

impl From<MutationKind> for Mutation {
    fn from(kind: MutationKind) -> Self {
        Self { kind }
    }
}

impl From<StatementInsertion> for Mutation {
    fn from(mutation: StatementInsertion) -> Self {
        MutationKind::StatementInsertion(mutation).into()
    }
}

impl From<StatementReplacement> for Mutation {
    fn from(mutation: StatementReplacement) -> Self {
        MutationKind::StatementReplacement(mutation).into()
    }
}
