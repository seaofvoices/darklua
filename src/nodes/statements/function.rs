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

    pub fn with_method<S: Into<String>>(mut self, method: S) -> Self {
        self.method.replace(method.into());
        self
    }

    pub fn push_field(&mut self, field: String) {
        self.field_names.push(field);
    }

    #[inline]
    pub fn remove_method(&mut self) -> Option<String> {
        self.method.take()
    }

    #[inline]
    pub fn get_method(&self) -> Option<&String> {
        self.method.as_ref()
    }

    #[inline]
    pub fn get_name(&self) -> &String {
        &self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[inline]
    pub fn get_field_names(&self) -> &Vec<String> {
        &self.field_names
    }

    #[inline]
    pub fn mutate_identifier(&mut self) -> &mut String {
        &mut self.name
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
    pub fn new(
        name: FunctionName,
        block: Block,
        parameters: Vec<String>,
        is_variadic: bool,
    ) -> Self {
        Self {
            name,
            block,
            parameters,
            is_variadic,
        }
    }

    pub fn from_name<S: Into<String>, B: Into<Block>>(name: S, block: B) -> Self {
        Self {
            name: FunctionName::from_name(name),
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

    #[inline]
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn get_name(&self) -> &FunctionName {
        &self.name
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
    pub fn mutate_function_name(&mut self) -> &mut FunctionName {
        &mut self.name
    }

    #[inline]
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
