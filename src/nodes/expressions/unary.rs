use crate::nodes::Expression;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Length,
    Minus,
    Not,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryExpression {
    operator: UnaryOperator,
    expression: Expression,
}

impl UnaryExpression {
    pub fn new<E: Into<Expression>>(operator: UnaryOperator, expression: E) -> Self {
        Self {
            operator,
            expression: expression.into(),
        }
    }

    #[inline]
    pub fn get_expression(&self) -> &Expression {
        &self.expression
    }

    #[inline]
    pub fn mutate_expression(&mut self) -> &mut Expression {
        &mut self.expression
    }

    #[inline]
    pub fn operator(&self) -> UnaryOperator {
        self.operator
    }
}
