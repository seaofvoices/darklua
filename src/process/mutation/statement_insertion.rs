use std::borrow::Borrow;

use crate::{
    nodes::Block,
    process::path::{NodePath, NodePathBuf, NodePathSlice},
};

use super::{MutationEffect, MutationResult, StatementInsertionContent};

#[derive(Clone, Debug)]
pub struct StatementInsertion {
    before_path: NodePathBuf,
    insertion: StatementInsertionContent,
}

impl StatementInsertion {
    pub fn insert_before(
        statement_path: impl Into<NodePathBuf>,
        insertion: impl Into<StatementInsertionContent>,
    ) -> Self {
        Self {
            before_path: statement_path.into(),
            insertion: insertion.into(),
        }
    }

    pub fn insert_after(
        statement_path: impl Borrow<NodePathSlice>,
        insertion: impl Into<StatementInsertionContent>,
    ) -> Self {
        let path = statement_path.borrow();

        let index = path
            .last_statement()
            .expect("path to insert statements must end with a statement component");

        let parent = path.parent().expect("path to insert statements must be ");

        Self {
            before_path: parent.join_statement(index + 1),
            insertion: insertion.into(),
        }
    }

    pub fn apply(self, block: &mut Block) -> MutationResult {
        let block_path = self.before_path.parent().ok_or_else(|| todo!())?;
        let index = self.before_path.statement_index().ok_or_else(|| todo!())?;

        let block = block_path.resolve_block_mut(block).ok_or_else(|| todo!())?;

        let mut effects = Vec::new();

        if !self.insertion.is_empty() {
            let length = self.insertion.len();
            self.insertion.apply(block, index);
            let path = block_path.join_statement(index);
            effects.push(MutationEffect::statement_added(path.clone().span(length)));
        }

        Ok(effects)
    }

    pub fn mutate(&mut self, effect: &MutationEffect) -> bool {
        let before_path = &self.before_path;
        match effect {
            MutationEffect::StatementRemoved(effect_span) => {
                if effect_span.contains(before_path) {
                    self.before_path = effect_span.path().clone();
                }
            }
            MutationEffect::StatementAdded(effect_span) => {
                let effect_path = effect_span.path();
                if effect_span.contains(before_path) {
                    self.before_path = effect_path.clone();
                } else if effect_path.parent() == before_path.parent() {
                    if let (Some(effect_index), Some(self_index)) =
                        (effect_path.last_statement(), before_path.last_statement())
                    {
                        let effect_end = effect_index.saturating_add(effect_path.len());
                        if effect_end <= self_index {
                            self.before_path = before_path
                                .parent()
                                .expect("todo: should have a parent")
                                .join_statement(self_index + effect_span.len());
                        }
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::process::mutation::StatementReplacement;
    use crate::process::path::NodePathBuf;

    fn statement_path(index: usize) -> NodePathBuf {
        NodePathBuf::default().with_statement(index)
    }

    macro_rules! test_insertion_before {
        ($($name:ident ( $path:expr, $code:literal, $insert:literal ) => $expect_code:literal => [ $($expect_effect:expr),* $(,)? ] ),* $(,)?) => {
            super::super::test::test_mutation!(
                $(
                    $name (
                        StatementInsertion::insert_before(
                            $path,
                            $crate::Parser::default()
                                .parse($insert)
                                .expect("given test code should parse")
                        ),
                    $code ) => $expect_code => [$( $expect_effect, )*],
                )*
            );
        };
        ($($name:ident ( $path:expr, $code:literal, $insert:literal ) => $expect_code:literal => $expect_effect:expr ),* $(,)?) => {
            test_replace!(
                $(
                    $name ( $path, $code, $insert ) => $expect_code => [$expect_effect],
                )*
            );
        };
    }

    test_insertion_before!(
        first_statement(
            statement_path(0),
            "return a + a",
            "local a = 1"
        )
            => "local a = 1 return a + a"
            => [
                MutationEffect::statement_added(statement_path(0))
            ],
    );

    mod statement_replace {
        use super::*;

        macro_rules! test_replace {
            ($($name:ident ( $path:expr, $code:literal, $insert:literal ) => $expect_code:literal => [ $($expect_effect:expr),* $(,)? ] ),* $(,)?) => {
                super::super::super::test::test_mutation!(
                    $(
                        $name (
                            StatementReplacement::replace(
                                $path,
                                $crate::Parser::default()
                                    .parse($insert)
                                    .expect("given test code should parse")
                            ),
                        $code ) => $expect_code => [$( $expect_effect, )*],
                    )*
                );
            };
            ($($name:ident ( $path:expr, $code:literal, $insert:literal ) => $expect_code:literal => $expect_effect:expr ),* $(,)?) => {
                test_replace!(
                    $(
                        $name ( $path, $code, $insert ) => $expect_code => [$expect_effect],
                    )*
                );
            };
        }

        test_replace!(
            replace_first_statement(
                statement_path(0),
                "local a = nil return a + a",
                "local a = 1"
            )
                => "local a = 1 return a + a"
                => [
                    MutationEffect::statement_removed(statement_path(0)),
                    MutationEffect::statement_added(statement_path(0))
                ],
            replace_first_statement_with_two_statements(
                statement_path(0),
                "local a = nil return a + a",
                "print('test') local a = 2"
            )
                => "print('test') local a = 2 return a + a"
                => [
                    MutationEffect::statement_removed(statement_path(0)),
                    MutationEffect::statement_added(statement_path(0).span(2))
                ],
            replace_two_first_statement_with_one_statement(
                statement_path(0).span(2),
                "local a = nil local b = variable return b",
                "local b = variable"
            )
                => "local b = variable return b"
                => [
                    MutationEffect::statement_removed(statement_path(0).span(2)),
                    MutationEffect::statement_added(statement_path(0))
                ],
            replace_last_statement(
                statement_path(1),
                "local a = true return a or a",
                "return true"
            )
                => "local a = true return true"
                => [
                    MutationEffect::statement_removed(statement_path(1)),
                    MutationEffect::statement_added(statement_path(1))
                ],
            replace_single_last_statement_with_statement(
                statement_path(0),
                "return true",
                "local t = {}"
            )
                => "local t = {}"
                => [
                    MutationEffect::statement_removed(statement_path(0)),
                    MutationEffect::statement_added(statement_path(0))
                ],
            replace_last_statement_with_statement(
                statement_path(1),
                "local a = 'upper' return {}",
                "local b = string[a]"
            )
                => "local a = 'upper' local b = string[a]"
                => [
                    MutationEffect::statement_removed(statement_path(1)),
                    MutationEffect::statement_added(statement_path(1))
                ],
        );
    }
}
