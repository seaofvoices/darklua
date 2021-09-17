use crate::nodes::{
    Expression,
    StringExpression,
    TableExpression,
};

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
}

impl Default for Arguments {
    fn default() -> Self {
        Arguments::Tuple(Vec::new())
    }
}

impl From<TableExpression> for Arguments {
    fn from(table: TableExpression) -> Self {
        Self::Table(table)
    }
}

impl From<StringExpression> for Arguments {
    fn from(string: StringExpression) -> Self {
        Self::String(string)
    }
}
