use crate::nodes::{Expression, Identifier, Token};

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

    pub fn clear_comments(&mut self) {
        self.field.clear_comments();
        if let Some(token) = &mut self.token {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.field.clear_whitespaces();
        if let Some(token) = &mut self.token {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.field.replace_referenced_tokens(code);
        if let Some(token) = &mut self.token {
            token.replace_referenced_tokens(code);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIndexEntryTokens {
    pub opening_bracket: Token,
    pub closing_bracket: Token,
    pub equal: Token,
}

impl TableIndexEntryTokens {
    pub fn clear_comments(&mut self) {
        self.opening_bracket.clear_comments();
        self.closing_bracket.clear_comments();
        self.equal.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_bracket.clear_whitespaces();
        self.closing_bracket.clear_whitespaces();
        self.equal.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_bracket.replace_referenced_tokens(code);
        self.closing_bracket.replace_referenced_tokens(code);
        self.equal.replace_referenced_tokens(code);
    }
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

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEntry {
    Field(TableFieldEntry),
    Index(TableIndexEntry),
    Value(Expression),
}

impl TableEntry {
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
    pub fn clear_comments(&mut self) {
        self.opening_brace.clear_comments();
        self.closing_brace.clear_comments();
        self.separators.iter_mut().for_each(Token::clear_comments);
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_brace.clear_whitespaces();
        self.closing_brace.clear_whitespaces();
        self.separators
            .iter_mut()
            .for_each(Token::clear_whitespaces);
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_brace.replace_referenced_tokens(code);
        self.closing_brace.replace_referenced_tokens(code);
        for separator in self.separators.iter_mut() {
            separator.replace_referenced_tokens(code);
        }
    }
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

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
        self.entries.iter_mut().for_each(TableEntry::clear_comments)
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
        self.entries
            .iter_mut()
            .for_each(TableEntry::clear_whitespaces)
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
        for entry in self.entries.iter_mut() {
            entry.replace_referenced_tokens(code);
        }
    }
}

impl Default for TableExpression {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
