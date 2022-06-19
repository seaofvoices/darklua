use crate::nodes::{Block, LastStatement, Statement};

use super::path::NodePath;

#[derive(Clone, Debug)]
pub struct StatementSpan {
    path: NodePath,
    length: usize,
}

impl StatementSpan {
    pub fn path(&self) -> &NodePath {
        &self.path
    }

    pub fn len(&self) -> usize {
        self.length + 1
    }
}

impl NodePath {
    pub fn span(self, length: usize) -> StatementSpan {
        debug_assert!(length > 0, "StatementSpan length must always be above zero");
        StatementSpan { path: self, length }
    }
}

impl From<NodePath> for StatementSpan {
    fn from(path: NodePath) -> Self {
        Self { path, length: 1 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MutationError {}

pub trait Mutation {
    fn apply(&self, block: &mut Block) -> Result<(), MutationError>;
}

#[derive(Clone, Debug, Default)]
pub struct StatementInsertion {
    statements: Vec<Statement>,
    last_statement: Option<LastStatement>,
}

#[derive(Clone, Debug)]
pub enum StatementMutation {
    Replace(StatementSpan, StatementInsertion),
    Insert(StatementSpan, StatementInsertion),
}

impl StatementMutation {
    pub fn remove(statement_span: impl Into<StatementSpan>) -> Self {
        Self::Replace(statement_span.into(), Default::default())
    }
}

impl Mutation for StatementMutation {
    fn apply(&self, block: &mut Block) -> Result<(), MutationError> {
        match self {
            StatementMutation::Replace(span, statements) => {
                let block_path = span.path().parent().ok_or_else(|| todo!())?;
                let index = span.path().statement_index().ok_or_else(|| todo!())?;

                let block = block_path.resolve_block_mut(block).ok_or_else(|| todo!())?;

                if block.total_len() == 0 {
                    todo!()
                }

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
            StatementMutation::Insert(_, _) => todo!(),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        );
    }
}
