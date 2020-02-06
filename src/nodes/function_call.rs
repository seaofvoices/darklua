use crate::lua_generator::{LuaGenerator, ToLua};
use crate::nodes::{
    Arguments,
    Prefix,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionCall {
    prefix: Box<Prefix>,
    arguments: Arguments,
    method: Option<String>,
}

impl FunctionCall {
    pub fn new(prefix: Prefix, arguments: Arguments, method: Option<String>) -> Self {
        Self {
            prefix: Box::new(prefix),
            arguments,
            method,
        }
    }

    pub fn from_name<T: Into<String>>(name: T) -> Self {
        Self {
            prefix: Box::new(Prefix::Identifier(name.into())),
            arguments: Arguments::Tuple(Vec::new()),
            method: None,
        }
    }

    pub fn with_arguments(mut self, arguments: Arguments) -> Self {
        self.arguments = arguments;
        self
    }

    pub fn with_method(mut self, method: String) -> Self {
        self.method.replace(method);
        self
    }

    pub fn get_arguments(&self) -> &Arguments {
        &self.arguments
    }

    pub fn get_method(&self) -> Option<&String> {
        self.method.as_ref()
    }

    pub fn get_prefix(&self) -> &Prefix {
        &self.prefix
    }

    pub fn take_method(&mut self) -> Option<String> {
        self.method.take()
    }

    pub fn set_arguments(&mut self, arguments: Arguments) {
        self.arguments = arguments;
    }

    pub fn set_method(&mut self, method: String) {
        self.method.replace(method);
    }

    pub fn mutate_arguments(&mut self) -> &mut Arguments {
        &mut self.arguments
    }

    pub fn mutate_prefix(&mut self) -> &mut Prefix {
        &mut self.prefix
    }
}

impl ToLua for FunctionCall {
    fn to_lua(&self, generator: &mut LuaGenerator) {
        self.prefix.to_lua(generator);

        if let Some(method) = &self.method {
            generator.push_char(':');
            generator.push_str(&method);
        }

        self.arguments.to_lua(generator);
    }
}
