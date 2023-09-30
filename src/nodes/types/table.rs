use crate::nodes::{Identifier, Token};

use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIndexerType {
    key_type: Box<Type>,
    value_type: Box<Type>,
    tokens: Option<TableIndexerTypeTokens>,
}

impl TableIndexerType {
    pub fn new(key_type: impl Into<Type>, value_type: impl Into<Type>) -> Self {
        Self {
            key_type: Box::new(key_type.into()),
            value_type: Box::new(value_type.into()),
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

    pub fn with_tokens(mut self, token: TableIndexerTypeTokens) -> Self {
        self.tokens = Some(token);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, token: TableIndexerTypeTokens) {
        self.tokens = Some(token);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&TableIndexerTypeTokens> {
        self.tokens.as_ref()
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

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableIndexerTypeTokens {
    pub opening_bracket: Token,
    pub closing_bracket: Token,
    pub colon: Token,
}

impl TableIndexerTypeTokens {
    pub fn clear_comments(&mut self) {
        self.opening_bracket.clear_comments();
        self.closing_bracket.clear_comments();
        self.colon.clear_comments();
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_bracket.clear_whitespaces();
        self.closing_bracket.clear_whitespaces();
        self.colon.clear_whitespaces();
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_bracket.replace_referenced_tokens(code);
        self.closing_bracket.replace_referenced_tokens(code);
        self.colon.replace_referenced_tokens(code);
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.opening_bracket.shift_token_line(amount);
        self.closing_bracket.shift_token_line(amount);
        self.colon.shift_token_line(amount);
    }
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

    pub fn clear_comments(&mut self) {
        self.property.clear_comments();
        if let Some(token) = &mut self.token {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.property.clear_whitespaces();
        if let Some(token) = &mut self.token {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.property.replace_referenced_tokens(code);
        if let Some(token) = &mut self.token {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.property.shift_token_line(amount);
        if let Some(token) = &mut self.token {
            token.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableType {
    properties: Vec<TablePropertyType>,
    indexer: Option<TableIndexerType>,
    tokens: Option<TableTypeTokens>,
}

impl TableType {
    pub fn with_new_property(
        mut self,
        property: impl Into<Identifier>,
        r#type: impl Into<Type>,
    ) -> Self {
        self.properties
            .push(TablePropertyType::new(property.into(), r#type.into()));
        self
    }

    pub fn with_property(mut self, property: TablePropertyType) -> Self {
        self.properties.push(property);
        self
    }

    pub fn push_property(&mut self, property: TablePropertyType) {
        self.properties.push(property);
    }

    pub fn with_indexer_type(mut self, indexer_type: TableIndexerType) -> Self {
        self.indexer = Some(indexer_type);
        self
    }

    #[inline]
    pub fn set_indexer_type(&mut self, indexer_type: TableIndexerType) {
        self.indexer = Some(indexer_type);
    }

    #[inline]
    pub fn has_indexer_type(&self) -> bool {
        self.indexer.is_some()
    }

    #[inline]
    pub fn properties_len(&self) -> usize {
        self.properties.len()
    }

    #[inline]
    pub fn iter_property_type(&self) -> impl Iterator<Item = &TablePropertyType> {
        self.properties.iter()
    }

    #[inline]
    pub fn iter_mut_property_type(&mut self) -> impl Iterator<Item = &mut TablePropertyType> {
        self.properties.iter_mut()
    }

    #[inline]
    pub fn property_len(&self) -> usize {
        self.properties.len()
    }

    #[inline]
    pub fn get_indexer_type(&self) -> Option<&TableIndexerType> {
        self.indexer.as_ref()
    }

    #[inline]
    pub fn mutate_indexer_type(&mut self) -> Option<&mut TableIndexerType> {
        self.indexer.as_mut()
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

    pub fn clear_comments(&mut self) {
        for property in self.properties.iter_mut() {
            property.clear_comments();
        }
        if let Some(indexer) = &mut self.indexer {
            indexer.clear_comments();
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        for property in self.properties.iter_mut() {
            property.clear_whitespaces();
        }
        if let Some(indexer) = &mut self.indexer {
            indexer.clear_whitespaces();
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        for property in self.properties.iter_mut() {
            property.replace_referenced_tokens(code);
        }
        if let Some(indexer) = &mut self.indexer {
            indexer.replace_referenced_tokens(code);
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        for property in self.properties.iter_mut() {
            property.shift_token_line(amount);
        }
        if let Some(indexer) = &mut self.indexer {
            indexer.shift_token_line(amount);
        }
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableTypeTokens {
    pub opening_brace: Token,
    pub closing_brace: Token,
    pub separators: Vec<Token>,
}

impl TableTypeTokens {
    pub fn clear_comments(&mut self) {
        self.opening_brace.clear_comments();
        self.closing_brace.clear_comments();
        for token in self.separators.iter_mut() {
            token.clear_comments();
        }
    }

    pub fn clear_whitespaces(&mut self) {
        self.opening_brace.clear_whitespaces();
        self.closing_brace.clear_whitespaces();
        for token in self.separators.iter_mut() {
            token.clear_whitespaces();
        }
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.opening_brace.replace_referenced_tokens(code);
        self.closing_brace.replace_referenced_tokens(code);
        for token in self.separators.iter_mut() {
            token.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.opening_brace.shift_token_line(amount);
        self.closing_brace.shift_token_line(amount);
        for token in self.separators.iter_mut() {
            token.shift_token_line(amount);
        }
    }
}
