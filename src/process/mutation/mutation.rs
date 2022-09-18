use crate::nodes::Block;

use super::{
    MutationEffect, MutationResult, StatementInsertion, StatementInsertionContent,
    StatementReplacement, StatementSpan,
};

#[derive(Clone, Debug)]
pub enum Mutation {
    StatementInsertion(StatementInsertion),
    StatementReplacement(StatementReplacement),
}

impl Mutation {
    #[inline]
    pub fn remove(statement_span: impl Into<StatementSpan>) -> Self {
        Self::StatementReplacement(StatementReplacement::remove(statement_span))
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
            Self::StatementReplacement(StatementReplacement::replace(statement_span, insertion))
        }
    }

    pub fn apply(self, block: &mut Block) -> MutationResult {
        match self {
            Self::StatementInsertion(mutation) => mutation.apply(block),
            Self::StatementReplacement(mutation) => mutation.apply(block),
        }
    }

    /// Apply a given effect to the mutation and return true if the mutation
    /// should be kept or false if it should be discarded.
    pub fn mutate(&mut self, effect: &MutationEffect) -> bool {
        match self {
            Self::StatementInsertion(mutation) => mutation.mutate(effect),
            Self::StatementReplacement(mutation) => mutation.mutate(effect),
        }
    }
}

impl From<StatementInsertion> for Mutation {
    fn from(mutation: StatementInsertion) -> Self {
        Self::StatementInsertion(mutation)
    }
}

impl From<StatementReplacement> for Mutation {
    fn from(mutation: StatementReplacement) -> Self {
        Self::StatementReplacement(mutation)
    }
}
