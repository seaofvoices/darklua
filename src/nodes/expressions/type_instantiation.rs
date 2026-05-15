use crate::nodes::{Prefix, Token, Type};

/// Contains token information for an index expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInstantiationTokens {
    /// The first opening angle bracket token in the double `<<`.
    pub first_opening_list: Token,
    /// The second opening angle bracket token in the double `<<`.
    pub second_opening_list: Token,
    /// The first closing angle bracket token in the double `>>`.
    pub first_closing_list: Token,
    /// The second closing angle bracket token in the double `>>`.
    pub second_closing_list: Token,
    /// The comma tokens.
    pub commas: Vec<Token>,
}

impl TypeInstantiationTokens {
    super::impl_token_fns!(
        target = [first_opening_list, second_opening_list, first_closing_list, second_closing_list]
        iter = [commas]
    );
}

/// Represents a table index access expression.
///
/// An index expression accesses a value in a table using square bracket notation,
/// such as `table[key]`. It consists of a prefix (the table being accessed)
/// and an index expression that evaluates to the key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInstantiationExpression {
    prefix: Prefix,
    types: Vec<Type>,
    tokens: Option<TypeInstantiationTokens>,
}

impl TypeInstantiationExpression {
    /// Creates a new index expression with the given prefix and index expression.
    pub fn new(prefix: impl Into<Prefix>, types: Vec<Type>) -> Self {
        Self {
            prefix: prefix.into(),
            types,
            tokens: None,
        }
    }

    /// Attaches tokens to this index expression.
    pub fn with_tokens(mut self, tokens: TypeInstantiationTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Attaches tokens to this index expression.
    pub fn set_tokens(&mut self, tokens: TypeInstantiationTokens) {
        self.tokens = Some(tokens);
    }

    /// Returns a reference to the tokens attached to this index expression, if any.
    pub fn get_tokens(&self) -> Option<&TypeInstantiationTokens> {
        self.tokens.as_ref()
    }

    /// Returns a reference to the prefix of this index expression.
    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    /// Returns a reference to the index expression of this index expression.
    pub fn iter_types(&self) -> impl Iterator<Item = &Type> {
        self.types.iter()
    }

    pub fn types_len(&self) -> usize {
        self.types.len()
    }

    /// Returns a mutable reference to the prefix of this index expression.
    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }

    /// Returns a mutable reference to the index expression of this index expression.
    pub fn iter_mut_types(&mut self) -> impl Iterator<Item = &mut Type> {
        self.types.iter_mut()
    }

    /// Returns a mutable reference to the first token of this index expression,
    /// creating it if missing.
    pub fn mutate_first_token(&mut self) -> &mut crate::nodes::Token {
        self.prefix.mutate_first_token()
    }

    /// Returns a mutable reference to the last token of this index expression,
    /// creating it if missing.
    pub fn mutate_last_token(&mut self) -> &mut Token {
        if self.tokens.is_none() {
            self.tokens = Some(TypeInstantiationTokens {
                first_opening_list: Token::from_content("<"),
                second_opening_list: Token::from_content("<"),
                first_closing_list: Token::from_content(">"),
                second_closing_list: Token::from_content(">"),
                commas: (0..self.types.len().saturating_sub(1))
                    .map(|_| Token::from_content(","))
                    .collect(),
            });
        }
        &mut self.tokens.as_mut().unwrap().second_closing_list
    }

    super::impl_token_fns!(iter = [tokens]);
}
