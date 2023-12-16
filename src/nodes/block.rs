use crate::nodes::{LastStatement, ReturnStatement, Statement, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockTokens {
    pub semicolons: Vec<Option<Token>>,
    pub last_semicolon: Option<Token>,
    pub final_token: Option<Token>,
}

impl BlockTokens {
    pub fn clear_comments(&mut self) {
        for semicolon in self.semicolons.iter_mut().flatten() {
            semicolon.clear_comments();
        }
        if let Some(last_semicolon) = &mut self.last_semicolon {
            last_semicolon.clear_comments();
        }
        if let Some(final_token) = &mut self.final_token {
            final_token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        for semicolon in self.semicolons.iter_mut().flatten() {
            semicolon.clear_whitespaces();
        }
        if let Some(last_semicolon) = &mut self.last_semicolon {
            last_semicolon.clear_whitespaces();
        }
        if let Some(final_token) = &mut self.final_token {
            final_token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        for semicolon in self.semicolons.iter_mut().flatten() {
            semicolon.replace_referenced_tokens(code);
        }
        if let Some(last_semicolon) = &mut self.last_semicolon {
            last_semicolon.replace_referenced_tokens(code);
        }
        if let Some(final_token) = &mut self.final_token {
            final_token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        for semicolon in self.semicolons.iter_mut().flatten() {
            semicolon.shift_token_line(amount);
        }
        if let Some(last_semicolon) = &mut self.last_semicolon {
            last_semicolon.shift_token_line(amount);
        }
        if let Some(final_token) = &mut self.final_token {
            final_token.shift_token_line(amount);
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

    #[inline]
    pub fn mutate_tokens(&mut self) -> Option<&mut BlockTokens> {
        self.tokens.as_mut().map(|tokens| tokens.as_mut())
    }

    pub fn push_statement<T: Into<Statement>>(&mut self, statement: T) {
        if let Some(tokens) = &mut self.tokens {
            if self.statements.len() == tokens.semicolons.len() {
                tokens.semicolons.push(None);
            }
        }
        self.statements.push(statement.into());
    }

    pub fn with_statement<T: Into<Statement>>(mut self, statement: T) -> Self {
        self.statements.push(statement.into());
        self
    }

    pub fn insert_statement(&mut self, index: usize, statement: impl Into<Statement>) {
        if index > self.statements.len() {
            self.push_statement(statement.into());
        } else {
            self.statements.insert(index, statement.into());

            if let Some(tokens) = &mut self.tokens {
                if index <= tokens.semicolons.len() {
                    tokens.semicolons.insert(index, None);
                }
            }
        }
    }

    #[inline]
    pub fn set_last_statement(&mut self, last_statement: impl Into<LastStatement>) {
        self.last_statement = Some(last_statement.into());
    }

    pub fn with_last_statement(mut self, last_statement: impl Into<LastStatement>) -> Self {
        self.last_statement = Some(last_statement.into());
        self
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.last_statement.is_none() && self.statements.is_empty()
    }

    #[inline]
    pub fn statements_len(&self) -> usize {
        self.statements.len()
    }

    #[inline]
    pub fn iter_statements(&self) -> impl Iterator<Item = &Statement> {
        self.statements.iter()
    }

    #[inline]
    pub fn reverse_iter_statements(&self) -> impl Iterator<Item = &Statement> {
        self.statements.iter().rev()
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

    #[inline]
    pub fn first_statement(&self) -> Option<&Statement> {
        self.statements.first()
    }

    #[inline]
    pub fn first_mut_statement(&mut self) -> Option<&mut Statement> {
        self.statements.first_mut()
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

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
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
    use crate::{
        nodes::{DoStatement, RepeatStatement},
        Parser,
    };

    fn parse_block_with_tokens(lua: &str) -> Block {
        let parser = Parser::default().preserve_tokens();
        parser.parse(lua).expect("code should parse")
    }

    fn parse_statement_with_tokens(lua: &str) -> Statement {
        let mut block = parse_block_with_tokens(lua);
        assert!(block.get_last_statement().is_none());
        let statements = block.take_statements();
        assert_eq!(statements.len(), 1);
        statements.into_iter().next().unwrap()
    }

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
        assert_eq!(block.get_last_statement(), None);
    }

    #[test]
    fn set_last_statement() {
        let mut block = Block::default();
        let continue_statement = LastStatement::new_continue();
        block.set_last_statement(continue_statement.clone());

        assert_eq!(block.get_last_statement(), Some(&continue_statement));
    }

    #[test]
    fn insert_statement_at_index_0() {
        let mut block = Block::default().with_statement(DoStatement::default());

        let new_statement = RepeatStatement::new(Block::default(), false);
        block.insert_statement(0, new_statement.clone());

        assert_eq!(
            block,
            Block::default()
                .with_statement(new_statement)
                .with_statement(DoStatement::default())
        );
    }

    #[test]
    fn insert_statement_at_index_0_with_tokens() {
        let mut block = parse_block_with_tokens("do end;");

        block.insert_statement(0, RepeatStatement::new(Block::default(), false));

        insta::assert_debug_snapshot!("insert_statement_at_index_0_with_tokens", block);
    }

    #[test]
    fn insert_statement_at_upper_bound() {
        let mut block = Block::default().with_statement(DoStatement::default());

        let new_statement = RepeatStatement::new(Block::default(), false);
        block.insert_statement(1, new_statement.clone());

        assert_eq!(
            block,
            Block::default()
                .with_statement(DoStatement::default())
                .with_statement(new_statement)
        );
    }

    #[test]
    fn insert_statement_after_statement_upper_bound() {
        let mut block = Block::default().with_statement(DoStatement::default());

        let new_statement = RepeatStatement::new(Block::default(), false);
        block.insert_statement(4, new_statement.clone());

        assert_eq!(
            block,
            Block::default()
                .with_statement(DoStatement::default())
                .with_statement(new_statement)
        );
    }

    #[test]
    fn insert_statement_after_statement_upper_bound_with_tokens() {
        let mut block = parse_block_with_tokens("do end;");

        block.insert_statement(4, RepeatStatement::new(Block::default(), false));

        insta::assert_debug_snapshot!(
            "insert_statement_after_statement_upper_bound_with_tokens",
            block
        );
    }

    #[test]
    fn push_statement_with_tokens() {
        let mut block = parse_block_with_tokens("");

        let new_statement = parse_statement_with_tokens("while true do end");
        block.push_statement(new_statement);

        pretty_assertions::assert_eq!(
            block.get_tokens(),
            Some(&BlockTokens {
                semicolons: vec![None],
                last_semicolon: None,
                final_token: None,
            })
        );
    }

    #[test]
    fn clean_removes_semicolon_tokens() {
        let mut block = Block::default()
            .with_statement(DoStatement::default())
            .with_tokens(BlockTokens {
                semicolons: vec![Some(Token::from_content(";"))],
                last_semicolon: None,
                final_token: None,
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
                final_token: None,
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
                final_token: None,
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
                final_token: None,
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
                final_token: None,
            });

        block.filter_statements(|_statement| false);

        pretty_assertions::assert_eq!(
            block,
            Block::default().with_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: None,
                final_token: None,
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
                final_token: None,
            });

        block.filter_mut_statements(|_statement| false);

        pretty_assertions::assert_eq!(
            block,
            Block::default().with_tokens(BlockTokens {
                semicolons: Vec::new(),
                last_semicolon: None,
                final_token: None,
            })
        );
    }
}
