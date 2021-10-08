use crate::nodes::{Expression, StringExpression, TableExpression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TupleArgumentsTokens {
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
    pub commas: Vec<Token>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TupleArguments {
    values: Vec<Expression>,
    tokens: Option<TupleArgumentsTokens>,
}

impl TupleArguments {
    pub fn new(values: Vec<Expression>) -> Self {
        Self {
            values,
            tokens: None,
        }
    }

    #[inline]
    pub fn to_expressions(self) -> Vec<Expression> {
        self.values
    }

    pub fn with_tokens(mut self, tokens: TupleArgumentsTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: TupleArgumentsTokens) {
        self.tokens = Some(tokens);
    }

    pub fn with_argument<T: Into<Expression>>(mut self, argument: T) -> Self {
        self.values.push(argument.into());
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[inline]
    pub fn iter_values(&self) -> impl Iterator<Item = &Expression> {
        self.values.iter()
    }

    #[inline]
    pub fn iter_mut_values(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.values.iter_mut()
    }
}

impl Default for TupleArguments {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            tokens: None,
        }
    }
}

impl From<Arguments> for TupleArguments {
    fn from(arguments: Arguments) -> Self {
        match arguments {
            Arguments::Tuple(tuple) => tuple,
            Arguments::String(string) => TupleArguments::default().with_argument(string),
            Arguments::Table(table) => TupleArguments::default().with_argument(table),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arguments {
    Tuple(TupleArguments),
    String(StringExpression),
    Table(TableExpression),
}

impl Arguments {
    pub fn to_expressions(self) -> Vec<Expression> {
        match self {
            Self::Tuple(expressions) => expressions.to_expressions(),
            Self::String(string) => vec![string.into()],
            Self::Table(table) => vec![table.into()],
        }
    }

    pub fn with_argument<T: Into<Expression>>(self, argument: T) -> Self {
        TupleArguments::from(self).with_argument(argument).into()
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Self::Tuple(TupleArguments::default())
    }
}

impl From<TupleArguments> for Arguments {
    fn from(tuple: TupleArguments) -> Self {
        Self::Tuple(tuple)
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
