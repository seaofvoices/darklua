use crate::{
    nodes::{Expression, Identifier, Token, Trivia},
    process::utils::is_valid_identifier,
};

use super::StringExpression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableFieldEntry {
    field: Identifier,
    value: Expression,
    /// The token for the `=` operator symbol.
    token: Option<Token>,
}

impl TableFieldEntry {
    pub fn new<I: Into<Identifier>, E: Into<Expression>>(field: I, value: E) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    #[inline]
    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    #[inline]
    pub fn get_token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    #[inline]
    pub fn get_field(&self) -> &Identifier {
        &self.field
    }

    #[inline]
    pub fn mutate_field(&mut self) -> &mut Identifier {
        &mut self.field
    }

    #[inline]
    pub fn get_value(&self) -> &Expression {
        &self.value
    }

    #[inline]
    pub fn mutate_value(&mut self) -> &mut Expression {
        &mut self.value
    }

    super::impl_token_fns!(
        target = [field]
        iter = [token]
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIndexEntryTokens {
    pub opening_bracket: Token,
    pub closing_bracket: Token,
    pub equal: Token,
}

impl TableIndexEntryTokens {
    super::impl_token_fns!(target = [opening_bracket, closing_bracket, equal]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIndexEntry {
    key: Expression,
    value: Expression,
    tokens: Option<Box<TableIndexEntryTokens>>,
}

impl TableIndexEntry {
    pub fn new<T: Into<Expression>, U: Into<Expression>>(key: T, value: U) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: TableIndexEntryTokens) -> Self {
        self.tokens = Some(tokens.into());
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: TableIndexEntryTokens) {
        self.tokens = Some(tokens.into());
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TableIndexEntryTokens> {
        self.tokens.as_ref().map(|tokens| tokens.as_ref())
    }

    #[inline]
    pub fn get_key(&self) -> &Expression {
        &self.key
    }

    #[inline]
    pub fn mutate_key(&mut self) -> &mut Expression {
        &mut self.key
    }

    #[inline]
    pub fn get_value(&self) -> &Expression {
        &self.value
    }

    #[inline]
    pub fn mutate_value(&mut self) -> &mut Expression {
        &mut self.value
    }

    super::impl_token_fns!(iter = [tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEntry {
    Field(TableFieldEntry),
    Index(TableIndexEntry),
    Value(Expression),
}

impl TableEntry {
    /// Creates a field entry if the provided key is a valid identifier, otherwise it
    /// creates an index entry.
    pub fn from_string_key_and_value(key: impl Into<String>, value: impl Into<Expression>) -> Self {
        let key = key.into();
        let value = value.into();
        if is_valid_identifier(&key) {
            Self::Field(TableFieldEntry {
                field: Identifier::new(key),
                value,
                token: None,
            })
        } else {
            Self::Index(TableIndexEntry {
                key: Expression::String(StringExpression::from_value(key)),
                value,
                tokens: None,
            })
        }
    }

    pub fn clear_comments(&mut self) {
        match self {
            TableEntry::Field(entry) => entry.clear_comments(),
            TableEntry::Index(entry) => entry.clear_comments(),
            TableEntry::Value(_) => {}
        }
    }

    pub fn clear_whitespaces(&mut self) {
        match self {
            TableEntry::Field(entry) => entry.clear_whitespaces(),
            TableEntry::Index(entry) => entry.clear_whitespaces(),
            TableEntry::Value(_) => {}
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        match self {
            TableEntry::Field(entry) => entry.replace_referenced_tokens(code),
            TableEntry::Index(entry) => entry.replace_referenced_tokens(code),
            TableEntry::Value(_) => {}
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: isize) {
        match self {
            TableEntry::Field(entry) => entry.shift_token_line(amount),
            TableEntry::Index(entry) => entry.shift_token_line(amount),
            TableEntry::Value(_) => {}
        }
    }

    pub(crate) fn filter_comments(&mut self, filter: impl Fn(&Trivia) -> bool) {
        match self {
            TableEntry::Field(entry) => entry.filter_comments(filter),
            TableEntry::Index(entry) => entry.filter_comments(filter),
            TableEntry::Value(_) => {}
        }
    }
}

impl From<TableFieldEntry> for TableEntry {
    fn from(entry: TableFieldEntry) -> Self {
        Self::Field(entry)
    }
}

impl From<TableIndexEntry> for TableEntry {
    fn from(entry: TableIndexEntry) -> Self {
        Self::Index(entry)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableTokens {
    pub opening_brace: Token,
    pub closing_brace: Token,
    pub separators: Vec<Token>,
}

impl TableTokens {
    super::impl_token_fns!(
        target = [opening_brace, closing_brace]
        iter = [separators]
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableExpression {
    entries: Vec<TableEntry>,
    tokens: Option<TableTokens>,
}

impl TableExpression {
    pub fn new(entries: Vec<TableEntry>) -> Self {
        Self {
            entries,
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: TableTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: TableTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TableTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn get_entries(&self) -> &Vec<TableEntry> {
        &self.entries
    }

    #[inline]
    pub fn iter_entries(&self) -> impl Iterator<Item = &TableEntry> {
        self.entries.iter()
    }

    #[inline]
    pub fn iter_mut_entries(&mut self) -> impl Iterator<Item = &mut TableEntry> {
        self.entries.iter_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn mutate_entries(&mut self) -> &mut Vec<TableEntry> {
        &mut self.entries
    }

    pub fn append_entry<T: Into<TableEntry>>(mut self, entry: T) -> Self {
        self.entries.push(entry.into());
        self
    }

    pub fn append_field<S: Into<Identifier>, E: Into<Expression>>(
        mut self,
        key: S,
        value: E,
    ) -> Self {
        self.entries.push(TableFieldEntry::new(key, value).into());
        self
    }

    pub fn append_index<T: Into<Expression>, U: Into<Expression>>(
        mut self,
        key: T,
        value: U,
    ) -> Self {
        self.entries
            .push(TableIndexEntry::new(key.into(), value.into()).into());
        self
    }

    pub fn append_array_value<E: Into<Expression>>(mut self, value: E) -> Self {
        self.entries.push(TableEntry::Value(value.into()));
        self
    }

    super::impl_token_fns!(iter = [tokens, entries]);
}

impl Default for TableExpression {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
