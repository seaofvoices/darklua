use crate::nodes::{LastStatement, ReturnStatement, Statement, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockTokens {
    pub semicolons: Vec<Option<Token>>,
    pub last_semicolon: Option<Token>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    statements: Vec<Statement>,
    last_statement: Option<LastStatement>,
    tokens: Option<Box<BlockTokens>>,
}

impl Block {
    pub fn new(statements: Vec<Statement>, last_statement: Option<LastStatement>) -> Self {
        Self {
            statements,
            last_statement,
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: BlockTokens) -> Self {
        self.tokens = Some(tokens.into());
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: BlockTokens) {
        self.tokens = Some(tokens.into());
    }

    pub fn with_statement<T: Into<Statement>>(mut self, statement: T) -> Self {
        self.statements.push(statement.into());
        self
    }

    pub fn with_last_statement(mut self, last_statement: LastStatement) -> Self {
        self.last_statement = Some(last_statement);
        self
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.last_statement.is_none() && self.statements.is_empty()
    }

    #[inline]
    pub fn get_statements(&self) -> &Vec<Statement> {
        &self.statements
    }

    #[inline]
    pub fn get_last_statement(&self) -> Option<&LastStatement> {
        self.last_statement.as_ref()
    }

    pub fn filter_statements<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Statement) -> bool,
    {
        let mut i = 0;

        while i != self.statements.len() {
            if f(&mut self.statements[i]) {
                i += 1;
            } else {
                self.statements.remove(i);
            }
        }
    }

    #[inline]
    pub fn mutate_statements(&mut self) -> &mut Vec<Statement> {
        &mut self.statements
    }

    #[inline]
    pub fn mutate_last_statement(&mut self) -> &mut Option<LastStatement> {
        &mut self.last_statement
    }

    pub fn clear(&mut self) {
        self.statements.clear();
        self.last_statement.take();
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new(Vec::new(), None)
    }
}

impl<IntoStatement: Into<Statement>> From<IntoStatement> for Block {
    fn from(statement: IntoStatement) -> Block {
        Block::new(vec![statement.into()], None)
    }
}

impl From<LastStatement> for Block {
    fn from(statement: LastStatement) -> Block {
        Block::new(Vec::new(), Some(statement))
    }
}

impl From<ReturnStatement> for Block {
    fn from(statement: ReturnStatement) -> Block {
        Block::new(Vec::new(), Some(statement.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nodes::DoStatement;

    #[test]
    fn default_block_is_empty() {
        let block = Block::default();

        assert!(block.is_empty());
    }

    #[test]
    fn is_empty_is_true_when_block_has_no_statements_or_last_statement() {
        let block = Block::new(Vec::new(), None);

        assert!(block.is_empty());
    }

    #[test]
    fn is_empty_is_false_when_block_has_a_last_statement() {
        let block = Block::default().with_last_statement(LastStatement::new_break());

        assert!(!block.is_empty());
    }

    #[test]
    fn is_empty_is_false_when_block_a_statement() {
        let block = Block::default().with_statement(DoStatement::default());

        assert!(!block.is_empty());
    }

    #[test]
    fn clear_removes_statements() {
        let mut block = Block::default().with_statement(DoStatement::default());
        block.clear();

        assert!(block.is_empty());
    }

    #[test]
    fn clear_removes_last_statement() {
        let mut block = Block::default().with_last_statement(LastStatement::new_break());
        block.clear();

        assert!(block.is_empty());
    }
}
