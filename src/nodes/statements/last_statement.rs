use crate::nodes::Expression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LastStatement {
    Break,
    Return(Vec<Expression>),
}
