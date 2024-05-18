use crate::nodes::{BinaryOperator, Expression, Token, Variable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompoundOperator {
    Plus,
    Minus,
    Asterisk,
    Slash,
    DoubleSlash,
    Percent,
    Caret,
    Concat,
}

impl CompoundOperator {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Plus => "+=",
            Self::Minus => "-=",
            Self::Asterisk => "*=",
            Self::Slash => "/=",
            Self::DoubleSlash => "//=",
            Self::Percent => "%=",
            Self::Caret => "^=",
            Self::Concat => "..=",
        }
    }

    pub fn to_binary_operator(&self) -> BinaryOperator {
        match self {
            Self::Plus => BinaryOperator::Plus,
            Self::Minus => BinaryOperator::Minus,
            Self::Asterisk => BinaryOperator::Asterisk,
            Self::Slash => BinaryOperator::Slash,
            Self::DoubleSlash => BinaryOperator::DoubleSlash,
            Self::Percent => BinaryOperator::Percent,
            Self::Caret => BinaryOperator::Caret,
            Self::Concat => BinaryOperator::Concat,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundAssignTokens {
    pub operator: Token,
}

impl CompoundAssignTokens {
    super::impl_token_fns!(target = [operator]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundAssignStatement {
    operator: CompoundOperator,
    variable: Variable,
    value: Expression,
    tokens: Option<CompoundAssignTokens>,
}

impl CompoundAssignStatement {
    pub fn new<V: Into<Variable>, E: Into<Expression>>(
        operator: CompoundOperator,
        variable: V,
        value: E,
    ) -> Self {
        Self {
            operator,
            variable: variable.into(),
            value: value.into(),
            tokens: None,
        }
    }

    pub fn with_tokens(mut self, tokens: CompoundAssignTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: CompoundAssignTokens) {
        self.tokens = Some(tokens);
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&CompoundAssignTokens> {
        self.tokens.as_ref()
    }

    #[inline]
    pub fn get_operator(&self) -> CompoundOperator {
        self.operator
    }

    #[inline]
    pub fn get_variable(&self) -> &Variable {
        &self.variable
    }

    #[inline]
    pub fn get_value(&self) -> &Expression {
        &self.value
    }

    #[inline]
    pub fn extract_assignment(self) -> (Variable, Expression) {
        (self.variable, self.value)
    }

    #[inline]
    pub fn mutate_variable(&mut self) -> &mut Variable {
        &mut self.variable
    }

    #[inline]
    pub fn mutate_value(&mut self) -> &mut Expression {
        &mut self.value
    }

    super::impl_token_fns!(iter = [tokens]);
}
