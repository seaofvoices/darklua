use crate::nodes::Block;

use super::{MutationEffect, MutationResult, StatementInsertion, StatementReplacement};

#[derive(Clone, Debug)]
pub enum Mutation {
    StatementReplacement(StatementReplacement),
    StatementInsertion(StatementInsertion),
}

impl Mutation {
    pub fn apply(self, block: &mut Block) -> MutationResult {
        match self {
            Self::StatementReplacement(mutation) => mutation.apply(block),
            Self::StatementInsertion(mutation) => mutation.apply(block),
        }
    }

    /// Apply a given effect to the mutation and return true if the mutation
    /// should be kept or false if it should be discarded.
    pub fn mutate(&mut self, effect: &MutationEffect) -> bool {
        match self {
            Self::StatementReplacement(mutation) => mutation.mutate(effect),
            Self::StatementInsertion(mutation) => mutation.mutate(effect),
        }
    }
}

impl From<StatementReplacement> for Mutation {
    fn from(mutation: StatementReplacement) -> Self {
        Self::StatementReplacement(mutation)
    }
}

impl From<StatementInsertion> for Mutation {
    fn from(mutation: StatementInsertion) -> Self {
        Self::StatementInsertion(mutation)
    }
}
