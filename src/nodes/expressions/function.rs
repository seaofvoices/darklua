use crate::nodes::Block;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionExpression {
    block: Block,
    parameters: Vec<String>,
    is_variadic: bool,
}

impl FunctionExpression {
    pub fn new(block: Block, parameters: Vec<String>, is_variadic: bool) -> Self {
        Self {
            block,
            parameters,
            is_variadic,
        }
    }

    pub fn from_block<B: Into<Block>>(block: B) -> Self {
        Self {
            block: block.into(),
            parameters: Vec::new(),
            is_variadic: false,
        }
    }

    pub fn with_parameter<S: Into<String>>(mut self, parameter: S) -> Self {
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

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_parameters(&self) -> &Vec<String> {
        &self.parameters
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
    pub fn mutate_parameters(&mut self) -> &mut Vec<String> {
        &mut self.parameters
    }
}

impl Default for FunctionExpression {
    fn default() -> Self {
        Self {
            block: Block::default(),
            parameters: Vec::new(),
            is_variadic: false,
        }
    }
}
