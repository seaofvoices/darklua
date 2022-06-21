mod statement_insertion;
mod statement_span;

pub use statement_insertion::StatementInsertion;
pub use statement_span::StatementSpan;

use crate::nodes::Block;

use super::path::NodePath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MutationError {}

pub trait Mutation {
    fn apply(self, block: &mut Block) -> Result<(), MutationError>;
}

#[derive(Clone, Debug)]
pub struct StatementMutation {
    span: StatementSpan,
    insertion: StatementInsertion,
}

impl StatementMutation {
    pub fn remove(statement_span: impl Into<StatementSpan>) -> Self {
        Self {
            span: statement_span.into(),
            insertion: Default::default(),
        }
    }

    pub fn replace(
        statement_span: impl Into<StatementSpan>,
        insertion: impl Into<StatementInsertion>,
    ) -> Self {
        Self {
            span: statement_span.into(),
            insertion: insertion.into(),
        }
    }

    pub fn insert_before(
        statement_path: impl Into<NodePath>,
        insertion: impl Into<StatementInsertion>,
    ) -> Self {
        Self {
            span: statement_path.into().span(0),
            insertion: insertion.into(),
        }
    }

    pub fn insert_after(
        statement_path: impl Into<NodePath>,
        insertion: impl Into<StatementInsertion>,
    ) -> Self {
        Self {
            span: statement_path.into().span(0),
            insertion: insertion.into(),
        }
    }

    pub fn shift(&mut self) {
        todo!()
    }
}

impl Mutation for StatementMutation {
    fn apply(self, block: &mut Block) -> Result<(), MutationError> {
        let span = self.span;
        let path = span.path();
        let block_path = path.parent().ok_or_else(|| todo!())?;
        let index = path.statement_index().ok_or_else(|| todo!())?;

        let block = block_path.resolve_block_mut(block).ok_or_else(|| todo!())?;

        if block.total_len() == 0 {
            todo!()
        }

        if span.len() != 0 {
            let bound = index + span.len();
            let block_length = block.total_len();

            if index < block_length {
                let real_bound =
                    if block.has_last_statement() && block_length == bound.saturating_sub(1) {
                        block.take_last_statement();
                        bound.saturating_sub(1)
                    } else {
                        bound
                    };

                for remove_index in (index..real_bound).rev() {
                    block.remove_statement(remove_index);
                }
            } else {
                todo!()
            }
        }

        self.insertion.apply(block, index);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::process::path::NodePath;

    mod statement_removal {
        use super::*;

        macro_rules! test_removal {
            ($($name:ident ( $path:expr, $code:literal ) => $expect_code:literal ),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let parser = $crate::Parser::default();
                        let mut block = parser
                            .parse($code)
                            .expect("given test code should parse");

                        let expected_block = parser
                            .parse($expect_code)
                            .expect("given test code should parse");

                        let mutation = StatementMutation::remove($path);

                        pretty_assertions::assert_eq!(mutation.apply(&mut block), Ok(()));
                        pretty_assertions::assert_eq!(block, expected_block);
                    }
                )*
            }
        }

        test_removal!(
            remove_first_statement(
                NodePath::default().with_statement(0),
                "do end return"
            ) => "return",
            remove_first_statement_2(
                NodePath::default().with_statement(0),
                "local function nothing() return end return nothing"
            ) => "return nothing",
            remove_last_statement(
                NodePath::default().with_statement(1),
                "do end return"
            ) => "do end",
            remove_only_statement(
                NodePath::default().with_statement(0),
                "do end"
            ) => "",
            remove_only_last_statement(
                NodePath::default().with_statement(0),
                "return"
            ) => "",
            remove_middle_statement(
                NodePath::default().with_statement(1),
                "local a = 1 local b = 2 return a"
            ) => "local a = 1 return a",
            remove_after_middle_statement(
                NodePath::default().with_statement(1).span(2),
                "local a = 1 local b = 2 return a"
            ) => "local a = 1",
            remove_statement_and_last_statement(
                NodePath::default().with_statement(0).span(2),
                "while true do end return a"
            ) => "",
            remove_all_two_statements(
                NodePath::default().with_statement(0).span(2),
                "local function print() end local function test() print() end"
            ) => "",
            remove_all_three_statement(
                NodePath::default().with_statement(0).span(3),
                "local a = 1 local b = 2 return a"
            ) => "",
        );
    }

    mod statement_replace {
        use super::*;

        macro_rules! test_replace {
            ($($name:ident ( $path:expr, $code:literal, $insert:literal ) => $expect_code:literal ),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let parser = $crate::Parser::default();
                        let mut block = parser
                            .parse($code)
                            .expect("given test code should parse");

                        let insert = parser
                            .parse($insert)
                            .expect("given test code should parse");

                        let expected_block = parser
                            .parse($expect_code)
                            .expect("given test code should parse");

                        let mutation = StatementMutation::replace($path, insert);

                        pretty_assertions::assert_eq!(mutation.apply(&mut block), Ok(()));
                        pretty_assertions::assert_eq!(block, expected_block);
                    }
                )*
            }
        }

        test_replace!(
            replace_first_statement(
                NodePath::default().with_statement(0),
                "local a = nil return a + a",
                "local a = 1"
            ) => "local a = 1 return a + a",
            replace_first_statement_with_two_statements(
                NodePath::default().with_statement(0),
                "local a = nil return a + a",
                "print('test') local a = 2"
            ) => "print('test') local a = 2 return a + a",
            replace_two_first_statement_with_one_statement(
                NodePath::default().with_statement(0).span(2),
                "local a = nil local b = variable return b",
                "local b = variable"
            ) => "local b = variable return b",
            replace_last_statement(
                NodePath::default().with_statement(1),
                "local a = true return a or a",
                "return true"
            ) => "local a = true return true",
        );
    }
}
