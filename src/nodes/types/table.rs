use crate::nodes::{Identifier, Token, Trivia};

use super::{StringType, Type};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIndexerType {
    key_type: Type,
    value_type: Type,
    tokens: Option<TableIndexTypeTokens>,
}

impl TableIndexerType {
    pub fn new(key_type: impl Into<Type>, value_type: impl Into<Type>) -> Self {
        Self {
            key_type: key_type.into(),
            value_type: value_type.into(),
            tokens: None,
        }
    }

    #[inline]
    pub fn get_key_type(&self) -> &Type {
        &self.key_type
    }

    #[inline]
    pub fn mutate_key_type(&mut self) -> &mut Type {
        &mut self.key_type
    }

    #[inline]
    pub fn get_value_type(&self) -> &Type {
        &self.value_type
    }

    #[inline]
    pub fn mutate_value_type(&mut self) -> &mut Type {
        &mut self.value_type
    }

    pub fn with_tokens(mut self, token: TableIndexTypeTokens) -> Self {
        self.tokens = Some(token);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, token: TableIndexTypeTokens) {
        self.tokens = Some(token);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TableIndexTypeTokens> {
        self.tokens.as_ref()
    }

    super::impl_token_fns!(iter = [tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIndexTypeTokens {
    pub opening_bracket: Token,
    pub closing_bracket: Token,
    pub colon: Token,
}

impl TableIndexTypeTokens {
    super::impl_token_fns!(target = [opening_bracket, closing_bracket, colon]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TablePropertyType {
    property: Identifier,
    r#type: Type,
    token: Option<Token>,
}

impl TablePropertyType {
    pub fn new(property: impl Into<Identifier>, r#type: impl Into<Type>) -> Self {
        Self {
            property: property.into(),
            r#type: r#type.into(),
            token: None,
        }
    }

    #[inline]
    pub fn get_identifier(&self) -> &Identifier {
        &self.property
    }

    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut Identifier {
        &mut self.property
    }

    #[inline]
    pub fn get_type(&self) -> &Type {
        &self.r#type
    }

    #[inline]
    pub fn mutate_type(&mut self) -> &mut Type {
        &mut self.r#type
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

    super::impl_token_fns!(target = [property] iter = [token]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableLiteralPropertyType {
    string: StringType,
    r#type: Type,
    tokens: Option<TableIndexTypeTokens>,
}

impl TableLiteralPropertyType {
    pub fn new(string: StringType, r#type: impl Into<Type>) -> Self {
        Self {
            string,
            r#type: r#type.into(),
            tokens: None,
        }
    }

    #[inline]
    pub fn get_string(&self) -> &StringType {
        &self.string
    }

    #[inline]
    pub fn mutate_string(&mut self) -> &mut StringType {
        &mut self.string
    }

    #[inline]
    pub fn get_type(&self) -> &Type {
        &self.r#type
    }

    #[inline]
    pub fn mutate_type(&mut self) -> &mut Type {
        &mut self.r#type
    }

    pub fn with_tokens(mut self, tokens: TableIndexTypeTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: TableIndexTypeTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TableIndexTypeTokens> {
        self.tokens.as_ref()
    }

    super::impl_token_fns!(target = [string] iter = [tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEntryType {
    Property(TablePropertyType),
    Literal(TableLiteralPropertyType),
    Indexer(TableIndexerType),
}

impl TableEntryType {
    pub fn clear_comments(&mut self) {
        match self {
            TableEntryType::Property(property) => property.clear_comments(),
            TableEntryType::Literal(literal) => literal.clear_comments(),
            TableEntryType::Indexer(indexer) => indexer.clear_comments(),
        }
    }

    pub fn clear_whitespaces(&mut self) {
        match self {
            TableEntryType::Property(property) => property.clear_whitespaces(),
            TableEntryType::Literal(literal) => literal.clear_whitespaces(),
            TableEntryType::Indexer(indexer) => indexer.clear_whitespaces(),
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        match self {
            TableEntryType::Property(property) => property.replace_referenced_tokens(code),
            TableEntryType::Literal(literal) => literal.replace_referenced_tokens(code),
            TableEntryType::Indexer(indexer) => indexer.replace_referenced_tokens(code),
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: isize) {
        match self {
            TableEntryType::Property(property) => property.shift_token_line(amount),
            TableEntryType::Literal(literal) => literal.shift_token_line(amount),
            TableEntryType::Indexer(indexer) => indexer.shift_token_line(amount),
        }
    }

    pub(crate) fn filter_comments(&mut self, filter: impl Fn(&Trivia) -> bool) {
        match self {
            TableEntryType::Property(property) => property.filter_comments(filter),
            TableEntryType::Literal(literal) => literal.filter_comments(filter),
            TableEntryType::Indexer(indexer) => indexer.filter_comments(filter),
        }
    }
}

impl From<TablePropertyType> for TableEntryType {
    fn from(value: TablePropertyType) -> Self {
        Self::Property(value)
    }
}

impl From<TableLiteralPropertyType> for TableEntryType {
    fn from(value: TableLiteralPropertyType) -> Self {
        Self::Literal(value)
    }
}

impl From<TableIndexerType> for TableEntryType {
    fn from(value: TableIndexerType) -> Self {
        Self::Indexer(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableType {
    entries: Vec<TableEntryType>,
    tokens: Option<TableTypeTokens>,
}

impl TableType {
    pub fn with_new_property(
        mut self,
        property: impl Into<Identifier>,
        r#type: impl Into<Type>,
    ) -> Self {
        self.entries
            .push(TablePropertyType::new(property.into(), r#type.into()).into());
        self
    }

    pub fn with_property(mut self, property: impl Into<TableEntryType>) -> Self {
        self.push_property(property.into());
        self
    }

    #[inline]
    pub fn push_property(&mut self, property: impl Into<TableEntryType>) {
        let property = property.into();
        match property {
            TableEntryType::Indexer(indexer) => {
                self.set_indexer_type(indexer);
            }
            _ => {
                self.entries.push(property);
            }
        }
    }

    pub fn with_indexer_type(mut self, indexer_type: TableIndexerType) -> Self {
        self.set_indexer_type(indexer_type);
        self
    }

    #[inline]
    pub fn set_indexer_type(&mut self, indexer_type: TableIndexerType) -> Option<TableIndexerType> {
        match indexer_type.key_type {
            Type::String(string_type) => {
                self.entries
                    .push(TableEntryType::Literal(TableLiteralPropertyType {
                        string: string_type,
                        r#type: indexer_type.value_type,
                        tokens: indexer_type.tokens,
                    }));
                None
            }
            _ => {
                let removed = if let Some((remove_index, _)) = self
                    .entries
                    .iter()
                    .enumerate()
                    .find(|(_, entry)| matches!(entry, TableEntryType::Indexer(_)))
                {
                    match self.entries.remove(remove_index) {
                        TableEntryType::Indexer(indexer) => Some(indexer),
                        TableEntryType::Property(_) | TableEntryType::Literal(_) => unreachable!(),
                    }
                } else {
                    None
                };
                self.entries.push(TableEntryType::Indexer(indexer_type));
                removed
            }
        }
    }

    #[inline]
    pub fn has_indexer_type(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry, TableEntryType::Indexer(_)))
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
    pub fn iter_entries(&self) -> impl Iterator<Item = &TableEntryType> {
        self.entries.iter()
    }

    #[inline]
    pub fn iter_mut_entries(&mut self) -> impl Iterator<Item = &mut TableEntryType> {
        self.entries.iter_mut()
    }

    pub fn with_tokens(mut self, tokens: TableTypeTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: TableTypeTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TableTypeTokens> {
        self.tokens.as_ref()
    }

    super::impl_token_fns!(iter = [entries, tokens]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableTypeTokens {
    pub opening_brace: Token,
    pub closing_brace: Token,
    pub separators: Vec<Token>,
}

impl TableTypeTokens {
    super::impl_token_fns!(
        target = [opening_brace, closing_brace]
        iter = [separators]
    );
}
