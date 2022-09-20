use crate::{nodes::Block, process::path::NodePath};

use super::{
    MutationEffect, MutationError, MutationResult, StatementInsertionContent, StatementSpan,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatementReplacement {
    span: StatementSpan,
    insertion: StatementInsertionContent,
}

impl StatementReplacement {
    pub fn remove(statement_span: impl Into<StatementSpan>) -> Self {
        Self {
            span: statement_span.into(),
            insertion: Default::default(),
        }
    }

    pub fn replace(
        statement_span: impl Into<StatementSpan>,
        insertion: impl Into<StatementInsertionContent>,
    ) -> Self {
        Self {
            span: statement_span.into(),
            insertion: insertion.into(),
        }
    }

    pub fn span(&self) -> &StatementSpan {
        &self.span
    }

    pub fn shift_forward(&mut self, statement_count: usize) {
        if let (Some(parent), Some(last)) =
            (self.span.path().parent(), self.span.path().last_statement())
        {
            let new_path = parent.join_statement(last.saturating_add(statement_count));
            self.span.set_path(new_path);
        }
    }

    pub fn shift_backward(&mut self, statement_count: usize) {
        if let (Some(parent), Some(last)) =
            (self.span.path().parent(), self.span.path().last_statement())
        {
            let new_path = parent.join_statement(last.saturating_sub(statement_count));
            self.span.set_path(new_path);
        }
    }

    pub fn apply(self, block: &mut Block) -> MutationResult {
        let span = &self.span;
        let path = span.path();
        let block_path = path.parent().ok_or_else(|| {
            MutationError::new(self.clone())
                .unexpected_path(path.to_path_buf())
                .context("path should have a parent")
        })?;
        let index = path.statement_index().ok_or_else(|| {
            MutationError::new(self.clone())
                .statement_path_expected(path.to_path_buf())
                .context("mutation path")
        })?;

        let block = block_path.resolve_block_mut(block).ok_or_else(|| {
            MutationError::new(self.clone())
                .block_path_expected(block_path.to_path_buf())
                .context("mutation path parent")
        })?;

        if block.total_len() == 0 {
            return Err(MutationError::new(self.clone())
                .unexpected_path(block_path)
                .context("block should not be empty"));
        }

        let mut effects = Vec::new();

        if span.len() != 0 {
            let bound = index + span.len();
            let block_length = block.total_len();

            if index < block_length {
                let real_bound = if block.has_last_statement() && block_length == bound {
                    block.take_last_statement();
                    bound.saturating_sub(1)
                } else {
                    bound
                };

                for remove_index in (index..real_bound).rev() {
                    block.remove_statement(remove_index);
                }

                effects.push(MutationEffect::statement_removed(span.clone()));
            } else {
                return Err(MutationError::new(self.clone())
                    .unexpected_path(path.to_path_buf())
                    .context(format!(
                        "statement index ({}) is out of block bound ({})",
                        index, block_length,
                    )));
            }
        }

        if !self.insertion.is_empty() {
            let length = self.insertion.len();
            self.insertion.apply(block, index);
            effects.push(MutationEffect::statement_added(path.clone().span(length)));
        }

        Ok(effects)
    }

    pub fn mutate(&mut self, effect: &MutationEffect) -> bool {
        match effect {
            MutationEffect::StatementRemoved(effect_span) => {
                if effect_span.contains_span(self.span()) {
                    return false;
                } else if self.span.contains_span(effect_span) {
                    let new_size = self.span.len() - effect_span.len();
                    if new_size == 0 {
                        return false;
                    }

                    self.span.resize(new_size);
                } else if effect_span.contains(self.span.path()) {
                    if self.insertion.is_empty() {
                        // this mutation is just removing statements, so it can still remove
                        // the part not removed from the effect
                        let effect_end = effect_span.last_path();

                        if let (Some(effect_index_end), Some(self_index_start)) = (
                            effect_end.last_statement(),
                            self.span.path().last_statement(),
                        ) {
                            let difference = (effect_index_end + 1) - self_index_start;

                            self.shift_backward(difference);
                            self.span.resize(self.span().len() - difference)
                        }
                    } else {
                        // this mutation is trying to replace statements where a part of the
                        // planned statements have been removed
                        return false;
                    }
                } else {
                    let self_last_path = self.span.last_path();
                    if effect_span.contains(&self_last_path) {
                        if self.insertion.is_empty() {
                            // this mutation is just removing statements, so it can still remove
                            // the part not removed from the effect

                            if let (Some(effect_index_start), Some(self_index_end)) = (
                                effect_span.path().last_statement(),
                                self_last_path.last_statement(),
                            ) {
                                let new_size =
                                    self.span.len() - (self_index_end - effect_index_start);

                                if new_size == 0 {
                                    return false;
                                }
                                self.span.resize(new_size);
                            }
                        } else {
                            // this mutation is trying to replace statements where a part of the
                            // planned statements have been removed
                            return false;
                        }
                    } else {
                        let self_path = self.span.path();
                        let effect_path = effect_span.path();

                        if effect_path.parent() == self_path.parent() {
                            if let (Some(effect_index), Some(self_index)) =
                                (effect_path.last_statement(), self_path.last_statement())
                            {
                                let effect_end = effect_index.saturating_add(effect_path.len());
                                if effect_end <= self_index {
                                    self.shift_backward(effect_span.len());
                                }
                            }
                        }
                    }
                }
            }
            MutationEffect::StatementAdded(effect_span) => {
                let self_path = self.span.path();
                let effect_path = effect_span.path();

                if effect_path.parent() == self_path.parent() {
                    if let (Some(effect_index), Some(self_index)) =
                        (effect_path.last_statement(), self_path.last_statement())
                    {
                        let effect_end_index = effect_index + effect_span.len().saturating_sub(1);
                        let self_end_index = self_index + self.span.len().saturating_sub(1);

                        if effect_end_index < self_index {
                            self.shift_forward(effect_span.len());
                        } else {
                            if effect_index <= self_index {
                                self.span.set_path(effect_span.next_path());
                            } else {
                                if effect_index <= self_end_index {
                                    self.span.resize(self.span.len() + effect_span.len());
                                }
                            }
                        }
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::process::path::NodePathBuf;

    fn statement_path(index: usize) -> NodePathBuf {
        NodePathBuf::default().with_statement(index)
    }

    macro_rules! test_removal {
        ($($name:ident ( $path:expr, $code:literal ) => $expect_code:literal => [ $($expect_effect:expr),* $(,)? ] ),* $(,)?) => {
            super::super::test::test_mutation!(
                $(
                    $name ( StatementReplacement::remove($path), $code ) => $expect_code => [$( $expect_effect, )*],
                )*
            );
        };
        ($($name:ident ( $path:expr, $code:literal ) => $expect_code:literal => $expect_effect:expr ),* $(,)?) => {
            test_removal!(
                $(
                    $name ( $path, $code ) => $expect_code => [$expect_effect],
                )*
            );
        };
    }

    test_removal!(
        remove_first_statement(statement_path(0), "do end return")
            => "return"
            => MutationEffect::statement_removed(statement_path(0)),
        remove_first_statement_local_function(
            statement_path(0),
            "local function nothing() return end return nothing"
        )
            => "return nothing"
            => MutationEffect::statement_removed(statement_path(0)),
        remove_last_statement(statement_path(1), "do end return")
            => "do end"
            => MutationEffect::statement_removed(statement_path(1)),
        remove_only_statement(statement_path(0), "do end")
            => ""
            => MutationEffect::statement_removed(statement_path(0)),
        remove_only_last_statement(
            statement_path(0),
            "return"
        )
            => ""
            => MutationEffect::statement_removed(statement_path(0)),
        remove_middle_statement(
            statement_path(1),
            "local a = 1 local b = 2 return a"
        )
            => "local a = 1 return a"
            => MutationEffect::statement_removed(statement_path(1)),
        remove_after_middle_statement(
            statement_path(1).span(2),
            "local a = 1 local b = 2 return a"
        )
            => "local a = 1"
            => MutationEffect::statement_removed(statement_path(1).span(2)),
        remove_statement_and_last_statement(
            statement_path(0).span(2),
            "while true do end return a"
        )
            => ""
            => MutationEffect::statement_removed(statement_path(0).span(2)),
        remove_all_two_statements(
            statement_path(0).span(2),
            "local function print() end local function test() print() end"
        )
            => ""
            => MutationEffect::statement_removed(statement_path(0).span(2)),
        remove_all_three_statement(
            statement_path(0).span(3),
            "local a = 1 local b = 2 return a"
        )
            => ""
            => MutationEffect::statement_removed(statement_path(0).span(3)),
    );
}
