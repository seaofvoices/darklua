use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::Block;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionName {
    name: String,
    field_names: Vec<String>,
    method: Option<String>,
}

impl FunctionName {
    pub fn new(name: String, field_names: Vec<String>, method: Option<String>) -> Self {
        Self {
            name,
            field_names,
            method,
        }
    }

    pub fn from_name<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            field_names: Vec::new(),
            method: None,
        }
    }

    pub fn with_fields(mut self, field_names: Vec<String>) -> Self {
        self.field_names = field_names;
        self
    }

    pub fn with_method(mut self, method: String) -> Self {
        self.method.replace(method);
        self
    }

    pub fn push_field(&mut self, field: String) {
        self.field_names.push(field);
    }

    pub fn remove_method(&mut self) -> Option<String> {
        self.method.take()
    }

    pub fn get_method(&mut self) -> &Option<String> {
        &self.method
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn mutate_identifier(&mut self) -> &mut String {
        &mut self.name
    }
}

impl ToLua for FunctionName {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str(&self.name);

        self.field_names.iter()
            .for_each(|field| {
                generator.push_char('.');
                generator.push_str(field);
            });

        if let Some(method_name) = &self.method {
            generator.push_char(':');
            generator.push_str(method_name);
        };
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionStatement {
    name: FunctionName,
    block: Block,
    parameters: Vec<String>,
    is_variadic: bool,
}

impl FunctionStatement {
    pub fn new(name: FunctionName, block: Block, parameters: Vec<String>, is_variadic: bool) -> Self {
        Self {
            name,
            block,
            parameters,
            is_variadic,
        }
    }

    pub fn from_name<S: Into<String>>(name: S, block: Block) -> Self {
        Self {
            name: FunctionName::from_name(name),
            block,
            parameters: Vec::new(),
            is_variadic: false,
        }
    }

    pub fn append_parameter(mut self, parameter: String) -> Self {
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

    pub fn mutate_function_name(&mut self) -> &mut FunctionName {
        &mut self.name
    }

    pub fn mutate_parameters(&mut self) -> &mut Vec<String> {
        &mut self.parameters
    }

    pub fn remove_method(&mut self) {
        if let Some(method_name) = self.name.remove_method() {
            self.name.push_field(method_name);
            self.parameters.insert(0, "self".to_owned());
        }
    }
}

impl ToLua for FunctionStatement {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        generator.push_str("function");
        self.name.to_lua(generator);
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
    fn generate_() {
        let output = FunctionStatement::from_name("foo", Block::default())
            .to_lua_string();

        assert_eq!(output, "function foo()end");
    }
}
