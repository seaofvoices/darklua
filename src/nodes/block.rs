use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::{LastStatement, Statement};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    statements: Vec<Statement>,
    last_statement: Option<LastStatement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>, last_statement: Option<LastStatement>) -> Self {
        Self {
            statements,
            last_statement,
        }
    }

    pub fn with_statement<T: Into<Statement>>(mut self, statement: T) -> Self {
        self.statements.push(statement.into());
        self
    }

    pub fn with_last_statement(mut self, last_statement: LastStatement) -> Self {
        self.last_statement = Some(last_statement);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.last_statement.is_none() && self.statements.is_empty()
    }

    pub fn get_statements(&self) -> &Vec<Statement> {
        &self.statements
    }

    pub fn filter_statements<F>(&mut self, mut f: F) where F: FnMut(&mut Statement) -> bool {
        let mut i = 0;

        while i != self.statements.len() {
            if f(&mut self.statements[i]) {
                i += 1;
            } else {
                self.statements.remove(i);
            }
        }
    }

    pub fn mutate_statements(&mut self) -> &mut Vec<Statement> {
        &mut self.statements
    }

    pub fn mutate_last_statement(&mut self) -> &mut Option<LastStatement> {
        &mut self.last_statement
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new(Vec::new(), None)
    }
}

impl ToLua for Block {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.for_each_and_between(
            &self.statements,
            |generator, statement| statement.to_lua(generator),
            |generator| generator.push_char(';'),
        );

        if let Some(last_statement) = &self.last_statement {
            if self.statements.len() > 0 {
                generator.push_char(';');
            };

            last_statement.to_lua(generator);
            generator.push_char(';');
        }
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
        let block = Block::default().with_last_statement(LastStatement::Break);

        assert!(!block.is_empty());
    }

    #[test]
    fn is_empty_is_false_when_block_a_statement() {
        let block = Block::default().with_statement(DoStatement::default());

        assert!(!block.is_empty());
    }
}
