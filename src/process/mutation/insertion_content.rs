use std::cmp::Ordering;

use crate::nodes::{Block, LastStatement, Statement};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatementInsertionContent {
    statements: Vec<Statement>,
    last_statement: Option<LastStatement>,
}

impl From<Block> for StatementInsertionContent {
    fn from(mut block: Block) -> Self {
        let last_statement = block.take_last_statement();
        let statements = block.take_statements();
        Self {
            statements,
            last_statement,
        }
    }
}

impl StatementInsertionContent {
    pub fn len(&self) -> usize {
        self.statements.len() + if self.last_statement.is_none() { 0 } else { 1 }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.last_statement.is_none() && self.statements.is_empty()
    }

    pub fn apply(self, block: &mut Block, mut index: usize) {
        for statement in self.statements {
            block.insert_statement(index, statement);
            index += 1;
        }

        if let Some(statement) = self.last_statement {
            match index.cmp(&block.total_len()) {
                Ordering::Less => todo!("truncate block"),
                Ordering::Equal => {
                    block.replace_last_statement(statement);
                }
                Ordering::Greater => todo!("push or error?"),
            }
        }
    }
}
