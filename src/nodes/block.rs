use std::cmp::Ordering;

use crate::nodes::{AnyStatementRef, LastStatement, ReturnStatement, Statement, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockTokens {
    pub semicolons: Vec<Option<Token>>,
    pub last_semicolon: Option<Token>,
}

impl BlockTokens {
    pub fn clear_comments(&mut self) {
        self.semicolons.iter_mut().for_each(|semicolon| {
            if let Some(semicolon) = semicolon {
                semicolon.clear_comments();
            }
        });
        if let Some(last_semicolon) = &mut self.last_semicolon {
            last_semicolon.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.semicolons.iter_mut().for_each(|semicolon| {
            if let Some(semicolon) = semicolon {
                semicolon.clear_whitespaces();
            }
        });
        if let Some(last_semicolon) = &mut self.last_semicolon {
            last_semicolon.clear_whitespaces();
        }
    }
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

    #[inline]
    pub fn get_tokens(&self) -> Option<&BlockTokens> {
        self.tokens.as_ref().map(|tokens| tokens.as_ref())
    }

    pub fn with_statement<T: Into<Statement>>(mut self, statement: T) -> Self {
        self.statements.push(statement.into());
        self
    }

    pub fn with_last_statement<T: Into<LastStatement>>(mut self, last_statement: T) -> Self {
        self.last_statement = Some(last_statement.into());
        self
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.last_statement.is_none() && self.statements.is_empty()
    }

    #[inline]
    pub fn total_len(&self) -> usize {
        self.statements.len() + if self.last_statement.is_some() { 1 } else { 0 }
    }

    #[inline]
    pub fn iter_statements(&self) -> impl Iterator<Item = &Statement> {
        self.statements.iter()
    }

    #[inline]
    pub fn get_last_statement(&self) -> Option<&LastStatement> {
        self.last_statement.as_ref()
    }

    pub fn filter_statements<F>(&mut self, mut f: F)
    where
        F: FnMut(&Statement) -> bool,
    {
        let mut i = 0;

        while i != self.statements.len() {
            if f(&self.statements[i]) {
                i += 1;
            } else {
                self.statements.remove(i);

                if let Some(tokens) = &mut self.tokens {
                    if i < tokens.semicolons.len() {
                        tokens.semicolons.remove(i);
                    }
                }
            }
        }
    }

    pub fn filter_mut_statements<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Statement) -> bool,
    {
        let mut i = 0;

        while i != self.statements.len() {
            if f(&mut self.statements[i]) {
                i += 1;
            } else {
                self.statements.remove(i);

                if let Some(tokens) = &mut self.tokens {
                    if i < tokens.semicolons.len() {
                        tokens.semicolons.remove(i);
                    }
                }
            }
        }
    }

    pub fn truncate(&mut self, length: usize) {
        self.statements.truncate(length);
        if let Some(tokens) = &mut self.tokens {
            tokens.semicolons.truncate(length);
        }
    }

    #[inline]
    pub fn iter_mut_statements(&mut self) -> impl Iterator<Item = &mut Statement> {
        self.statements.iter_mut()
    }

    pub fn take_statements(&mut self) -> Vec<Statement> {
        if let Some(tokens) = &mut self.tokens {
            tokens.semicolons.clear();
        }
        self.statements.drain(..).collect()
    }

    pub fn take_last_statement(&mut self) -> Option<LastStatement> {
        if let Some(tokens) = &mut self.tokens {
            tokens.last_semicolon.take();
        }
        self.last_statement.take()
    }

    pub fn set_statements(&mut self, statements: Vec<Statement>) {
        self.statements = statements;

        if let Some(tokens) = &mut self.tokens {
            tokens.semicolons.clear();
        }
    }

    #[inline]
    pub fn mutate_last_statement(&mut self) -> Option<&mut LastStatement> {
        self.last_statement.as_mut()
    }

    #[inline]
    pub fn replace_last_statement<S: Into<LastStatement>>(
        &mut self,
        statement: S,
    ) -> Option<LastStatement> {
        self.last_statement.replace(statement.into())
    }

    pub fn get_statement(&self, index: usize) -> Option<AnyStatementRef> {
        match index.cmp(&self.statements.len()) {
            Ordering::Less => self.statements.get(index).map(AnyStatementRef::from),
            Ordering::Equal => self.last_statement.as_ref().map(AnyStatementRef::from),
            Ordering::Greater => None,
        }
    }

    pub fn clear(&mut self) {
        self.statements.clear();
        self.last_statement.take();

        if let Some(tokens) = &mut self.tokens {
            tokens.semicolons.clear();
            tokens.last_semicolon = None;
        }
    }

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
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

    #[test]
    fn clean_removes_semicolon_tokens() {
        let mut block = Block::default()
            .with_statement(DoStatement::default())
            .with_tokens(BlockTokens {
                semicolons: vec![Some(Token::from_content(";"))],
                last_semicolon: None,
            });
        block.clear();

        assert!(block.get_tokens().unwrap().semicolons.is_empty());
    }

    #[test]
    fn clean_removes_last_semicolon_token() {
        let mut block = Block::default()
            .with_last_statement(LastStatement::new_break())
            .with_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: Some(Token::from_content(";")),
            });
        block.clear();

        assert!(block.get_tokens().unwrap().last_semicolon.is_none());
    }

    #[test]
    fn set_statements_clear_semicolon_tokens() {
        let mut block = Block::default()
            .with_statement(DoStatement::default())
            .with_tokens(BlockTokens {
                semicolons: vec![Some(Token::from_content(";"))],
                last_semicolon: None,
            });
        block.set_statements(Vec::new());

        assert!(block.get_tokens().unwrap().semicolons.is_empty());
    }

    #[test]
    fn take_last_statement_clear_semicolon_token() {
        let mut block = Block::default()
            .with_last_statement(LastStatement::new_break())
            .with_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: Some(Token::from_content(";")),
            });

        assert_eq!(
            block.take_last_statement(),
            Some(LastStatement::new_break())
        );

        assert!(block.get_tokens().unwrap().last_semicolon.is_none());
    }

    #[test]
    fn filter_statements_does_not_panic_when_semicolons_do_not_match() {
        let mut block = Block::default()
            .with_statement(DoStatement::default())
            .with_statement(DoStatement::default())
            .with_tokens(BlockTokens {
                semicolons: vec![Some(Token::from_content(";"))],
                last_semicolon: None,
            });

        block.filter_statements(|_statement| false);

        pretty_assertions::assert_eq!(
            block,
            Block::default().with_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: None,
            })
        );
    }

    #[test]
    fn filter_mut_statements_does_not_panic_when_semicolons_do_not_match() {
        let mut block = Block::default()
            .with_statement(DoStatement::default())
            .with_statement(DoStatement::default())
            .with_tokens(BlockTokens {
                semicolons: vec![Some(Token::from_content(";"))],
                last_semicolon: None,
            });

        block.filter_mut_statements(|_statement| false);

        pretty_assertions::assert_eq!(
            block,
            Block::default().with_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: None,
            })
        );
    }

    #[test]
    fn get_statement_of_empty_block_is_none() {
        let block = Block::default();

        assert_eq!(block.get_statement(0), None);
        assert_eq!(block.get_statement(2), None);
    }

    #[test]
    fn get_statement_outside_of_boundary() {
        let block = Block::default().with_statement(DoStatement::default());

        assert_eq!(block.get_statement(1), None);
    }

    #[test]
    fn get_statement_outside_of_boundary_with_last_statement() {
        let block = Block::default()
            .with_statement(DoStatement::default())
            .with_last_statement(LastStatement::new_break());

        assert_eq!(block.get_statement(2), None);
    }

    #[test]
    fn get_statement_returns_first_statement() {
        let block = Block::default().with_statement(DoStatement::default());

        assert_eq!(
            block.get_statement(0).unwrap(),
            AnyStatementRef::from(&Statement::Do(DoStatement::default().into()))
        );
    }

    #[test]
    fn get_statement_returns_first_when_it_is_the_last_statement() {
        let block = Block::default().with_last_statement(ReturnStatement::default());

        assert_eq!(
            block.get_statement(0).unwrap(),
            AnyStatementRef::from(&LastStatement::Return(ReturnStatement::default()))
        );
    }
}
