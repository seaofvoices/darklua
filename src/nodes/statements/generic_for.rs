use crate::nodes::{Block, Expression, Token, TypedIdentifier};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericForTokens {
    pub r#for: Token,
    pub r#in: Token,
    pub r#do: Token,
    pub end: Token,
    pub identifier_commas: Vec<Token>,
    pub value_commas: Vec<Token>,
}

impl GenericForTokens {
    pub fn clear_comments(&mut self) {
        self.r#for.clear_comments();
        self.r#in.clear_comments();
        self.r#do.clear_comments();
        self.end.clear_comments();
        self.identifier_commas
            .iter_mut()
            .chain(self.value_commas.iter_mut())
            .for_each(Token::clear_comments);
    }

    pub fn clear_whitespaces(&mut self) {
        self.r#for.clear_whitespaces();
        self.r#in.clear_whitespaces();
        self.r#do.clear_whitespaces();
        self.end.clear_whitespaces();
        self.identifier_commas
            .iter_mut()
            .chain(self.value_commas.iter_mut())
            .for_each(Token::clear_whitespaces);
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        self.r#for.replace_referenced_tokens(code);
        self.r#in.replace_referenced_tokens(code);
        self.r#do.replace_referenced_tokens(code);
        self.end.replace_referenced_tokens(code);
        for comma in self
            .identifier_commas
            .iter_mut()
            .chain(self.value_commas.iter_mut())
        {
            comma.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        self.r#for.shift_token_line(amount);
        self.r#in.shift_token_line(amount);
        self.r#do.shift_token_line(amount);
        self.end.shift_token_line(amount);
        for comma in self
            .identifier_commas
            .iter_mut()
            .chain(self.value_commas.iter_mut())
        {
            comma.shift_token_line(amount);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericForStatement {
    identifiers: Vec<TypedIdentifier>,
    expressions: Vec<Expression>,
    block: Block,
    tokens: Option<GenericForTokens>,
}

impl GenericForStatement {
    pub fn new<B: Into<Block>>(
        identifiers: Vec<TypedIdentifier>,
        expressions: Vec<Expression>,
        block: B,
    ) -> Self {
        Self {
            identifiers,
            expressions,
            block: block.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: GenericForTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: GenericForTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&GenericForTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_identifiers(&self) -> &Vec<TypedIdentifier> {
        &self.identifiers
    }

    #[inline]
    pub fn iter_identifiers(&self) -> impl Iterator<Item = &TypedIdentifier> {
        self.identifiers.iter()
    }

    #[inline]
    pub fn get_expressions(&self) -> &Vec<Expression> {
        &self.expressions
    }

    #[inline]
    pub fn iter_expressions(&self) -> impl Iterator<Item = &Expression> {
        self.expressions.iter()
    }

    #[inline]
    pub fn iter_mut_identifiers(&mut self) -> impl Iterator<Item = &mut TypedIdentifier> {
        self.identifiers.iter_mut()
    }

    #[inline]
    pub fn iter_mut_expressions(&mut self) -> impl Iterator<Item = &mut Expression> {
        self.expressions.iter_mut()
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn identifiers_len(&self) -> usize {
        self.identifiers.len()
    }

    #[inline]
    pub fn expressions_len(&self) -> usize {
        self.expressions.len()
    }

    pub fn clear_types(&mut self) {
        for identifier in &mut self.identifiers {
            identifier.remove_type();
        }
    }

    pub fn clear_comments(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_comments();
        }
        self.identifiers
            .iter_mut()
            .for_each(TypedIdentifier::clear_comments);
    }

    pub fn clear_whitespaces(&mut self) {
        if let Some(tokens) = &mut self.tokens {
            tokens.clear_whitespaces();
        }
        self.identifiers
            .iter_mut()
            .for_each(TypedIdentifier::clear_whitespaces);
    }

    pub(crate) fn replace_referenced_tokens(&mut self, code: &str) {
        if let Some(tokens) = &mut self.tokens {
            tokens.replace_referenced_tokens(code);
        }
        for identifier in self.identifiers.iter_mut() {
            identifier.replace_referenced_tokens(code);
        }
    }

    pub(crate) fn shift_token_line(&mut self, amount: usize) {
        if let Some(tokens) = &mut self.tokens {
            tokens.shift_token_line(amount);
        }
        for identifier in self.identifiers.iter_mut() {
            identifier.shift_token_line(amount);
        }
    }
}
