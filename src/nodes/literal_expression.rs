use std::{convert::TryFrom, num::FpCategory};

use crate::nodes::{
    BinaryNumber, DecimalNumber, Expression, HexNumber, Identifier, NumberExpression,
    StringExpression, TableEntry, TableExpression, TableFieldEntry, TableTokens, Token, Trivia,
};

/// A literal expression in Luau.
///
/// Literal expressions are used in function attribute arguments
/// (e.g., `@[attr(true, 42, "text")]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiteralExpression {
    /// The boolean literal `true`
    True(Option<Token>),
    /// The boolean literal `false`
    False(Option<Token>),
    /// The nil literal
    Nil(Option<Token>),
    /// A numeric literal (e.g., `42`, `3.14`)
    Number(NumberExpression),
    /// A string literal (e.g., `"hello"`, `[[text]]`)
    String(StringExpression),
    /// A table literal (e.g., `{ key = value }`)
    Table(Box<LiteralTable>),
}

impl LiteralExpression {
    /// Creates a new nil literal expression.
    pub fn nil() -> Self {
        Self::Nil(None)
    }
}

impl From<LiteralTable> for LiteralExpression {
    fn from(v: LiteralTable) -> Self {
        Self::Table(Box::new(v))
    }
}

impl From<StringExpression> for LiteralExpression {
    fn from(v: StringExpression) -> Self {
        Self::String(v)
    }
}

impl From<bool> for LiteralExpression {
    fn from(v: bool) -> Self {
        if v {
            Self::True(None)
        } else {
            Self::False(None)
        }
    }
}

impl TryFrom<f64> for LiteralExpression {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        match value.classify() {
            FpCategory::Nan => Err("NaN is not a valid literal expression"),
            FpCategory::Infinite => Err("Infinity is not a valid literal expression"),
            FpCategory::Zero => {
                Ok(DecimalNumber::new(if value.is_sign_positive() {
                    0.0
                } else {
                    // if literal expression are allowed to have negative values in the
                    // future, we should return -0.0 here
                    0.0
                })
                .into())
            }
            FpCategory::Subnormal | FpCategory::Normal => {
                if value < 0.0 {
                    Err("Negative numbers are not a valid literal expression")
                } else if value < 0.1 {
                    let exponent = value.log10().floor();

                    Ok(DecimalNumber::new(value)
                        .with_exponent(exponent as i64, true)
                        .into())
                } else if value > 999.0 && (value / 100.0).fract() == 0.0 {
                    let mut exponent = value.log10().floor();
                    let mut power = 10_f64.powf(exponent);

                    while exponent > 2.0 && (value / power).fract() != 0.0 {
                        exponent -= 1.0;
                        power /= 10.0;
                    }

                    Ok(DecimalNumber::new(value)
                        .with_exponent(exponent as i64, true)
                        .into())
                } else {
                    Ok(DecimalNumber::new(value).into())
                }
            }
        }
    }
}

impl TryFrom<f32> for LiteralExpression {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        LiteralExpression::try_from(value as f64)
    }
}

impl From<NumberExpression> for LiteralExpression {
    fn from(v: NumberExpression) -> Self {
        Self::Number(v)
    }
}

impl TryFrom<u64> for LiteralExpression {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        LiteralExpression::try_from(value as f64)
    }
}

impl From<u32> for LiteralExpression {
    fn from(value: u32) -> Self {
        LiteralExpression::try_from(value as f64)
            .expect("converting a u32 to a literal number expression should never fail")
    }
}

impl From<u16> for LiteralExpression {
    fn from(value: u16) -> Self {
        LiteralExpression::try_from(value as f64)
            .expect("converting a u16 to a literal number expression should never fail")
    }
}

impl From<u8> for LiteralExpression {
    fn from(value: u8) -> Self {
        LiteralExpression::try_from(value as f64)
            .expect("converting a u8 to a literal number expression should never fail")
    }
}

impl TryFrom<i64> for LiteralExpression {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        LiteralExpression::try_from(value as f64)
    }
}

impl TryFrom<i32> for LiteralExpression {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        LiteralExpression::try_from(value as f64)
    }
}

impl TryFrom<i16> for LiteralExpression {
    type Error = &'static str;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        LiteralExpression::try_from(value as f64)
    }
}

impl TryFrom<i8> for LiteralExpression {
    type Error = &'static str;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        LiteralExpression::try_from(value as f64)
    }
}

impl From<DecimalNumber> for LiteralExpression {
    fn from(number: DecimalNumber) -> Self {
        Self::Number(NumberExpression::Decimal(number))
    }
}

impl From<HexNumber> for LiteralExpression {
    fn from(number: HexNumber) -> Self {
        Self::Number(NumberExpression::Hex(number))
    }
}

impl From<BinaryNumber> for LiteralExpression {
    fn from(number: BinaryNumber) -> Self {
        Self::Number(NumberExpression::Binary(number))
    }
}

impl<T: Into<LiteralExpression>> From<Option<T>> for LiteralExpression {
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::nil(),
            Some(value) => value.into(),
        }
    }
}

impl From<LiteralExpression> for Expression {
    fn from(literal: LiteralExpression) -> Self {
        match literal {
            LiteralExpression::True(token) => Self::True(token),
            LiteralExpression::False(token) => Self::False(token),
            LiteralExpression::Nil(token) => Self::Nil(token),
            LiteralExpression::Number(num) => Self::Number(num),
            LiteralExpression::String(string) => Self::String(string),
            LiteralExpression::Table(table) => Self::Table((*table).into()),
        }
    }
}

/// A literal table in Luau.
///
/// Table literals follow the syntax `{ entry1, entry2, key = value }`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiteralTable {
    entries: Vec<LiteralTableEntry>,
    tokens: Option<TableTokens>,
}

impl LiteralTable {
    /// Creates a new literal table from a vector of entries.
    pub fn from_entries(entries: Vec<LiteralTableEntry>) -> Self {
        Self {
            entries,
            tokens: None,
        }
    }

    /// Adds an entry to this table.
    pub fn with_entry(mut self, entry: impl Into<LiteralTableEntry>) -> Self {
        self.entries.push(entry.into());
        self
    }

    /// Appends an entry to this table.
    pub fn append_entry(&mut self, entry: impl Into<LiteralTableEntry>) {
        self.entries.push(entry.into());
    }

    /// Appends a field entry to this table.
    pub fn append_field(&mut self, field: Identifier, value: impl Into<LiteralExpression>) {
        self.append_entry(LiteralTableEntry::Field(Box::new(LiteralTableFieldEntry {
            field,
            value: value.into(),
            token: None,
        })));
    }

    /// Appends an array value entry to this table.
    pub fn append_array_value(&mut self, value: impl Into<LiteralExpression>) {
        self.append_entry(LiteralTableEntry::Value(Box::new(value.into())));
    }

    /// Returns an iterator over the table entries.
    pub fn iter_entries(&self) -> impl Iterator<Item = &LiteralTableEntry> {
        self.entries.iter()
    }

    /// Returns a mutable iterator over the table entries.
    pub fn iter_mut_entries(&mut self) -> impl Iterator<Item = &mut LiteralTableEntry> {
        self.entries.iter_mut()
    }

    /// Attaches tokens to this table.
    pub fn with_tokens(mut self, tokens: TableTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Sets the tokens for this table.
    pub fn set_tokens(&mut self, tokens: TableTokens) {
        self.tokens = Some(tokens);
    }

    /// Returns the tokens for this table, if any.
    pub fn get_tokens(&self) -> Option<&TableTokens> {
        self.tokens.as_ref()
    }

    /// Returns the number of entries in this table.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this table is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    super::impl_token_fns!(iter = [tokens, entries]);
}

/// An entry in a literal table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiteralTableEntry {
    /// A named field entry (e.g., `{ field = value }`)
    Field(Box<LiteralTableFieldEntry>),
    /// A value entry for array-like tables (e.g., `{ value }`)
    Value(Box<LiteralExpression>),
}

impl LiteralTableEntry {
    /// Creates a value entry from a literal expression.
    pub fn from_value(value: impl Into<LiteralExpression>) -> Self {
        Self::Value(Box::new(value.into()))
    }

    /// Clears all comments from the tokens in this node.
    pub fn clear_comments(&mut self) {
        match self {
            Self::Field(entry) => entry.clear_comments(),
            Self::Value(_value) => {}
        }
    }

    /// Clears all whitespaces information from the tokens in this node.
    pub fn clear_whitespaces(&mut self) {
        match self {
            Self::Field(entry) => entry.clear_whitespaces(),
            Self::Value(_value) => {}
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        match self {
            Self::Field(entry) => entry.replace_referenced_tokens(code),
            Self::Value(_value) => {}
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: isize) {
        match self {
            Self::Field(entry) => entry.shift_token_line(amount),
            Self::Value(_value) => {}
        }
    }

    pub(crate) fn filter_comments(&mut self, filter: impl Fn(&Trivia) -> bool) {
        match self {
            Self::Field(entry) => entry.filter_comments(filter),
            Self::Value(_value) => {}
        }
    }
}

impl From<LiteralTableFieldEntry> for LiteralTableEntry {
    fn from(v: LiteralTableFieldEntry) -> Self {
        Self::Field(Box::new(v))
    }
}

impl From<LiteralExpression> for LiteralTableEntry {
    fn from(v: LiteralExpression) -> Self {
        Self::Value(Box::new(v))
    }
}

/// A field entry in a literal table.
///
/// Represents a named field assignment: `{ field = value }`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralTableFieldEntry {
    field: Identifier,
    value: LiteralExpression,
    token: Option<Token>,
}

impl LiteralTableFieldEntry {
    /// Attaches a token to this field entry for the `=` symbol.
    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    /// Sets the token for this field entry's `=` symbol.
    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    /// Returns the token for this field entry's `=` symbol, if any.
    pub fn get_token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    /// Returns the field name.
    pub fn get_field(&self) -> &Identifier {
        &self.field
    }

    /// Returns a mutable reference to the field name.
    pub fn mutate_field(&mut self) -> &mut Identifier {
        &mut self.field
    }

    /// Returns the field value.
    pub fn get_value(&self) -> &LiteralExpression {
        &self.value
    }

    /// Returns a mutable reference to the field value.
    pub fn mutate_value(&mut self) -> &mut LiteralExpression {
        &mut self.value
    }

    /// Returns a mutable reference to the token for this field entry's `=` symbol, if any.
    pub fn mutate_token(&mut self) -> Option<&mut Token> {
        self.token.as_mut()
    }

    super::impl_token_fns!(
        target = [field]
        iter = [token]
    );
}

impl From<LiteralTable> for TableExpression {
    fn from(literal_table: LiteralTable) -> Self {
        let entries = literal_table
            .entries
            .into_iter()
            .map(|entry| match entry {
                LiteralTableEntry::Field(field) => {
                    let field = *field;
                    TableEntry::Field(Box::new(TableFieldEntry::new(
                        field.field,
                        Expression::from(field.value),
                    )))
                }
                LiteralTableEntry::Value(value) => {
                    TableEntry::Value(Box::new(Expression::from(*value)))
                }
            })
            .collect();

        let mut table = TableExpression::new(entries);
        if let Some(tokens) = literal_table.tokens {
            table.set_tokens(tokens);
        }
        table
    }
}
