use crate::nodes::{Block, Identifier, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionExpressionTokens {
    pub function: Token,
    pub opening_parenthese: Token,
    pub closing_parenthese: Token,
    pub end: Token,
    pub parameter_commas: Vec<Token>,
    pub variable_arguments: Option<Token>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionExpression {
    block: Block,
    parameters: Vec<Identifier>,
    is_variadic: bool,
    tokens: Option<Box<FunctionExpressionTokens>>,
}

impl FunctionExpression {
    pub fn new(block: Block, parameters: Vec<Identifier>, is_variadic: bool) -> Self {
        Self {
            block,
            parameters,
            is_variadic,
            tokens: None,
        }
    }

    pub fn from_block<B: Into<Block>>(block: B) -> Self {
        Self {
            block: block.into(),
            parameters: Vec::new(),
            is_variadic: false,
            tokens: None,
        }
    }

    pub fn with_parameter<P: Into<Identifier>>(mut self, parameter: P) -> Self {
        self.parameters.push(parameter.into());
        self
    }

    pub fn variadic(mut self) -> Self {
        self.is_variadic = true;
        self
    }

    pub fn set_variadic(&mut self, is_variadic: bool) {
        self.is_variadic = is_variadic;
    }

    pub fn with_tokens(mut self, tokens: FunctionExpressionTokens) -> Self {
        self.tokens = Some(tokens.into());
        self
    }

    #[inline]
    pub fn set_tokens(&mut self, tokens: FunctionExpressionTokens) {
        self.tokens = Some(tokens.into());
    }

    #[inline]
    pub fn get_tokens(&self) -> Option<&FunctionExpressionTokens> {
        self.tokens.as_ref().map(|tokens| tokens.as_ref())
    }

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_parameters(&self) -> &Vec<Identifier> {
        &self.parameters
    }

    #[inline]
    pub fn iter_parameters(&self) -> impl Iterator<Item = &Identifier> {
        self.parameters.iter()
    }

    #[inline]
    pub fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn mutate_parameters(&mut self) -> &mut Vec<Identifier> {
        &mut self.parameters
    }

    #[inline]
    pub fn parameters_count(&self) -> usize {
        self.parameters.len()
    }

    #[inline]
    pub fn has_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }
}

impl Default for FunctionExpression {
    fn default() -> Self {
        Self {
            block: Block::default(),
            parameters: Vec::new(),
            is_variadic: false,
            tokens: None,
        }
    }
}
