use std::iter;

use crate::nodes::{Expression, StringExpression, TableExpression, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TupleArgumentsTokens {
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
    pub commas: Vec<Token>,
}

impl TupleArgumentsTokens {
    super::impl_token_fns!(
        target = [opening_parenthese, closing_parenthese]
        iter = [commas]
    );
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

    #[inline]
    pub fn get_tokens(&self) -> Option<&TupleArgumentsTokens> {
        self.tokens.as_ref()
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

    super::impl_token_fns!(iter = [tokens]);
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

impl iter::FromIterator<Expression> for TupleArguments {
    fn from_iter<T: IntoIterator<Item = Expression>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
            tokens: None,
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

    pub fn clear_comments(&mut self) {
        match self {
            Arguments::Tuple(tuple) => tuple.clear_comments(),
            Arguments::String(_) | Arguments::Table(_) => {}
        }
    }

    pub fn clear_whitespaces(&mut self) {
        match self {
            Arguments::Tuple(tuple) => tuple.clear_whitespaces(),
            Arguments::String(_) | Arguments::Table(_) => {}
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        match self {
            Arguments::Tuple(tuple) => tuple.replace_referenced_tokens(code),
            Arguments::String(_) | Arguments::Table(_) => {}
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: isize) {
        match self {
            Arguments::Tuple(tuple) => tuple.shift_token_line(amount),
            Arguments::String(_) | Arguments::Table(_) => {}
        }
    }

    pub(crate) fn filter_comments(&mut self, filter: impl Fn(&super::Trivia) -> bool) {
        match self {
            Arguments::Tuple(tuple) => tuple.filter_comments(filter),
            Arguments::String(_) | Arguments::Table(_) => {}
        }
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
