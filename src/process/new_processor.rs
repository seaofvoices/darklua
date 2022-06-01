use crate::nodes::{Block, Statement};

use super::mutations::StatementPath;

#[derive(Clone, Debug)]
pub struct Processor {
    block: Block,
}

impl Processor {
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    fn resolve(&self, path: StatementPath) -> Option<Statement> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tests_something() {

    }
}
