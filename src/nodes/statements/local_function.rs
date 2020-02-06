use crate::lua_generator::{LuaGenerator, ToLua};
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

    pub fn with_parameter(mut self, parameter: String) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn variadic(mut self) -> Self {
        self.is_variadic = true;
        self
    }

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }
}

impl ToLua for LocalFunctionStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("local");
        generator.push_str("function");
        generator.push_str(&self.identifier);
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
    fn generate_empty_function() {
        let output = LocalFunctionStatement::from_name("foo", Block::default())
            .to_lua_string();

        assert_eq!(output, "local function foo()end");
    }

    #[test]
    fn generate_empty_variadic_function() {
        let output = LocalFunctionStatement::from_name("foo", Block::default())
            .variadic()
            .to_lua_string();

        assert_eq!(output, "local function foo(...)end");
    }

    #[test]
    fn generate_empty_function_with_one_parameter() {
        let output = LocalFunctionStatement::from_name("foo", Block::default())
            .with_parameter("bar".to_owned())
            .to_lua_string();

        assert_eq!(output, "local function foo(bar)end");
    }

    #[test]
    fn generate_empty_function_with_two_parameters() {
        let output = LocalFunctionStatement::from_name("foo", Block::default())
            .with_parameter("bar".to_owned())
            .with_parameter("baz".to_owned())
            .to_lua_string();

        assert_eq!(output, "local function foo(bar,baz)end");
    }

    #[test]
    fn generate_empty_variadic_function_with_one_parameter() {
        let output = LocalFunctionStatement::from_name("foo", Block::default())
            .with_parameter("bar".to_owned())
            .variadic()
            .to_lua_string();

        assert_eq!(output, "local function foo(bar,...)end");
    }
}
