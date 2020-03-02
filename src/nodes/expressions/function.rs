use crate::lua_generator::{LuaGenerator, ToLua};
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

    pub fn append_parameter(mut self, parameter: String) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn set_variadic(mut self, is_variadic: bool) -> Self {
        self.is_variadic = is_variadic;
        self
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

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

impl ToLua for FunctionExpression {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("function");
        generator.push_char('(');
        generator.for_each_and_between(
            &self.parameters,
            |generator, parameter| generator.push_str(parameter),
            |generator| generator.push_char(','),
        );

        if self.is_variadic {
            if self.parameters.len() > 0 {
                generator.push_char(',');
            };
            generator.push_str("...");
        };

        generator.push_char(')');
        self.block.to_lua(generator);
        generator.push_str("end");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_empty_function_expression() {
        let output = FunctionExpression::default().to_lua_string();

        assert_eq!(output, "function()end");
    }

    #[test]
    fn generate_empty_variadic_function_expression() {
        let output = FunctionExpression::default()
            .set_variadic(true)
            .to_lua_string();

        assert_eq!(output, "function(...)end");
    }

    #[test]
    fn generate_empty_variadic_function_with_one_parameter() {
        let output = FunctionExpression::default()
            .append_parameter("a".to_owned())
            .set_variadic(true)
            .to_lua_string();

        assert_eq!(output, "function(a,...)end");
    }

    #[test]
    fn generate_empty_variadic_function_with_two_parameter() {
        let output = FunctionExpression::default()
            .append_parameter("a".to_owned())
            .append_parameter("b".to_owned())
            .set_variadic(true)
            .to_lua_string();

        assert_eq!(output, "function(a,b,...)end");
    }

    #[test]
    fn generate_empty_function_with_two_parameter() {
        let output = FunctionExpression::default()
            .append_parameter("a".to_owned())
            .append_parameter("b".to_owned())
            .to_lua_string();

        assert_eq!(output, "function(a,b)end");
    }
}
