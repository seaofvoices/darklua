use crate::nodes::{Block, Expression};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericForStatement {
    identifier: String,
    start: Expression,
    end: Expression,
    step: Option<Expression>,
    block: Block,
}

impl NumericForStatement {
    pub fn new<S: Into<String>, E1: Into<Expression>, E2: Into<Expression>, B: Into<Block>>(
        identifier: S,
        start: E1,
        end: E2,
        step: Option<Expression>,
        block: B,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            start: start.into(),
            end: end.into(),
            step,
            block: block.into(),
        }
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn get_start(&self) -> &Expression {
        &self.start
    }

    #[inline]
    pub fn mutate_start(&mut self) -> &mut Expression {
        &mut self.start
    }

    #[inline]
    pub fn get_end(&self) -> &Expression {
        &self.end
    }

    #[inline]
    pub fn mutate_end(&mut self) -> &mut Expression {
        &mut self.end
    }

    #[inline]
    pub fn get_step(&self) -> Option<&Expression> {
        self.step.as_ref()
    }

    #[inline]
    pub fn mutate_step(&mut self) -> &mut Option<Expression> {
        &mut self.step
    }

    #[inline]
    pub fn get_identifier(&self) -> &String {
        &self.identifier
    }

    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut String {
        &mut self.identifier
    }

    #[inline]
    pub fn set_identifier<S: Into<String>>(&mut self, identifier: S) {
        self.identifier = identifier.into();
    }
}
