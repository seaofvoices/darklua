use crate::nodes::Block;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalFunctionStatement {
    identifier: String,
    block: Block,
    parameters: Vec<String>,
    is_variadic: bool,
}

impl LocalFunctionStatement {
    pub fn new(
        identifier: String,
        block: Block,
        parameters: Vec<String>,
        is_variadic: bool,
    ) -> Self {
        Self {
            identifier,
            block,
            parameters,
            is_variadic,
        }
    }

    pub fn from_name<S: Into<String>>(identifier: S, block: Block) -> Self {
        Self {
            identifier: identifier.into(),
            block,
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

    #[inline]
    pub fn mutate_parameters(&mut self) -> &mut Vec<String> {
        &mut self.parameters
    }

    #[inline]
    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut String {
        &mut self.identifier
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
    pub fn get_name(&self) -> &str {
        &self.identifier
    }

    #[inline]
    pub fn has_parameter(&self, name: &str) -> bool {
        self.parameters.iter().any(|parameter| parameter == name)
    }

    #[inline]
    pub fn is_variadic(&self) -> bool {
        self.is_variadic
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn has_parameter_is_true_when_single_param_matches() {
        let func = LocalFunctionStatement::from_name("foo", Block::default())
            .with_parameter("bar");

        assert!(func.has_parameter("bar"));
    }

    #[test]
    fn has_parameter_is_true_when_at_least_one_param_matches() {
        let func = LocalFunctionStatement::from_name("foo", Block::default())
            .with_parameter("bar")
            .with_parameter("baz");

        assert!(func.has_parameter("baz"));
    }

    #[test]
    fn has_parameter_is_false_when_none_matches() {
        let func = LocalFunctionStatement::from_name("foo", Block::default())
            .with_parameter("bar")
            .with_parameter("baz");

        assert!(!func.has_parameter("foo"));
    }
}
