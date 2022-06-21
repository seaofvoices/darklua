use std::cmp::Ordering;

use crate::nodes::{Block, LastStatement, Statement};

#[derive(Clone, Debug, Default)]
pub struct StatementInsertion {
    statements: Vec<Statement>,
    last_statement: Option<LastStatement>,
}

impl From<Block> for StatementInsertion {
    fn from(mut block: Block) -> Self {
        let last_statement = block.take_last_statement();
        let statements = block.take_statements();
        Self {
            statements,
            last_statement,
        }
    }
}

impl StatementInsertion {
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
