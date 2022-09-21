use std::cmp::Ordering;

use crate::nodes::{Block, LastStatement, Statement};

use super::MutationError;

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

    pub fn apply(&self, block: &mut Block, mut index: usize) -> Result<(), MutationError> {
        for statement in &self.statements {
            block.insert_statement(index, statement.clone());
            index += 1;
        }

        if let Some(statement) = &self.last_statement {
            match index.cmp(&block.total_len()) {
                Ordering::Less => {
                    return Err(MutationError::default()
                        .invalid_insertion()
                        .context("unable to insert last statement within block statements"))
                }
                Ordering::Equal => {
                    block.replace_last_statement(statement.clone());
                }
                Ordering::Greater => {
                    if block.has_last_statement() {
                        return Err(MutationError::default()
                            .invalid_insertion()
                            .context("unable to insert last statement after an existing one"));
                    } else {
                        block.replace_last_statement(statement.clone());
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::Parser;

    macro_rules! test_insertion {
        ($($name:ident ( $code:literal, $insertion:literal, $index:expr ) => $expect_code:literal),* $(,)?) => {
            $(
                #[test]
                fn $name () {
                    let parser = $crate::Parser::default();
                    let mut block = parser
                        .parse($code)
                        .expect("given test code should parse");

                    let expected_block = parser
                        .parse($expect_code)
                        .expect("given test code should parse");

                    let insertion: StatementInsertionContent = Parser::default()
                        .parse($insertion)
                        .expect("given insertion code should parse").into();

                    insertion.apply(&mut block, $index).unwrap();

                    use $crate::generator::{LuaGenerator, ReadableLuaGenerator};

                    let mut generator = ReadableLuaGenerator::new(80);
                    generator.write_block(&block);

                    pretty_assertions::assert_eq!(
                        block,
                        expected_block,
                        "\nExpected: `{}`\nReceived: `{}`",
                        $expect_code,
                        generator.into_string(),
                    );
                }
            )*
        };
    }

    test_insertion!(
        one_statement("", "do end", 0) => "do end",
        one_statement_after_statement("local a", "a = 1", 1) => "local a a = 1",
        one_statement_before_statement("a = 1", "local a", 0) => "local a a = 1",
        one_last_statement("", "return", 0) => "return",
        one_last_statement_after_statement("local a", "return a", 1) => "local a return a",
        // insertions out of bound push at the end
        statement_out_of_bound("local a", "a = 1", 10)
            => "local a a = 1",
        last_statement_out_of_bound("local a", "return a", 10)
            => "local a return a",
    );

    macro_rules! test_insertion_failure {
        ($($name:ident ( $code:literal, $insertion:literal, $index:expr ) => $expect_error:expr),* $(,)?) => {
            $(
                #[test]
                fn $name () {
                    let parser = $crate::Parser::default();
                    let mut block = parser
                        .parse($code)
                        .expect("given test code should parse");

                    let insertion: StatementInsertionContent = Parser::default()
                        .parse($insertion)
                        .expect("given insertion code should parse").into();

                    pretty_assertions::assert_eq!(
                        insertion.apply(&mut block, $index).unwrap_err(),
                        $expect_error,
                    );
                }
            )*
        };
    }

    fn err() -> MutationError {
        MutationError::default().invalid_insertion()
    }

    fn last_statement_within_error() -> MutationError {
        err().context("unable to insert last statement within block statements")
    }

    test_insertion_failure!(
        last_statement_over_first_statement("local a", "return a", 0)
            => last_statement_within_error(),
        last_statement_over_only_last_statement("return a", "return true", 0)
            => last_statement_within_error(),
        last_statement_over_last_statement("local a return a", "return true", 1)
            => last_statement_within_error(),
        last_statement_out_of_bound_with_existing_last_statement("return a", "return true", 4)
            => err().context("unable to insert last statement after an existing one"),
    );
}
