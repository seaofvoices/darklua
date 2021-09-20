use crate::nodes::{Expression, Prefix};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexExpression {
    prefix: Prefix,
    index: Expression,
}

impl IndexExpression {
    pub fn new<E: Into<Expression>>(prefix: Prefix, expression: E) -> Self {
        Self {
            prefix,
            index: expression.into(),
        }
    }

    #[inline]
    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    #[inline]
    pub fn get_index(&self) -> &Expression {
        &self.index
    }

    #[inline]
    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }

    #[inline]
    pub fn mutate_index(&mut self) -> &mut Expression {
        &mut self.index
    }
}
