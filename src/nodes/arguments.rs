use crate::nodes::{
    Expression,
    StringExpression,
    TableExpression,
};

use std::mem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arguments {
    Tuple(Vec<Expression>),
    String(StringExpression),
    Table(TableExpression),
}

impl Arguments {
    pub fn to_expressions(self) -> Vec<Expression> {
        match self {
            Self::Tuple(expressions) => expressions,
            Self::String(string) => vec![string.into()],
            Self::Table(table) => vec![table.into()],
        }
    }

    pub fn append_argument<T: Into<Expression>>(self, argument: T) -> Self {
        let mut expressions = self.to_expressions();
        expressions.push(argument.into());
        Arguments::Tuple(expressions)
    }

    pub fn push_argument<T: Into<Expression>>(&mut self, argument: T) -> &mut Self {
        match self {
            Self::Tuple(expressions) => {
                expressions.push(argument.into());
            }
            Self::Table(_) | Self::String(_) => {
                match mem::replace(self, Self::Tuple(Vec::new())) {
                    Self::Table(table) => {
                        self.push_argument(table);
                    }
                    Self::String(string) => {
                        self.push_argument(string);
                    }
                    Self::Tuple(_) => {
                        unreachable!()
                    }
                }
                self.push_argument(argument);
            }
        }
        self
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Arguments::Tuple(Vec::new())
    }
}
