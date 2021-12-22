use crate::nodes::{
    Block, FunctionExpression, FunctionStatement, Identifier, LocalFunctionStatement,
};

use super::TupleValue;

pub type EngineFunctionImpl = fn(TupleValue) -> TupleValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionValue {
    Lua(LuaFunction),
    Engine(EngineFunction),
    Unknown,
}

impl From<EngineFunctionImpl> for FunctionValue {
    fn from(function: EngineFunctionImpl) -> Self {
        Self::Engine(EngineFunction::new(function))
    }
}

impl From<LuaFunction> for FunctionValue {
    fn from(function: LuaFunction) -> Self {
        Self::Lua(function)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaFunction {
    parameters: Vec<String>,
    is_variadic: bool,
    block: Box<Block>,
    parent_state: Option<usize>,
}

impl LuaFunction {
    pub fn new(block: Block, parameters: Vec<String>, is_variadic: bool) -> Self {
        Self {
            parameters,
            is_variadic,
            block: block.into(),
            parent_state: None,
        }
    }

    pub fn with_parent_state(mut self, state: usize) -> Self {
        self.parent_state = Some(state);
        self
    }

    pub fn mutate_block(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn iter_parameters(&self) -> impl Iterator<Item = &String> {
        self.parameters.iter()
    }
}

impl From<&LocalFunctionStatement> for LuaFunction {
    fn from(function: &LocalFunctionStatement) -> Self {
        Self::new(
            function.get_block().clone(),
            function
                .iter_parameters()
                .map(Identifier::get_name)
                .map(ToOwned::to_owned)
                .collect(),
            function.is_variadic(),
        )
    }
}

impl From<&FunctionExpression> for LuaFunction {
    fn from(function: &FunctionExpression) -> Self {
        Self::new(
            function.get_block().clone(),
            function
                .iter_parameters()
                .map(Identifier::get_name)
                .map(ToOwned::to_owned)
                .collect(),
            function.is_variadic(),
        )
    }
}

impl From<&FunctionStatement> for LuaFunction {
    fn from(function: &FunctionStatement) -> Self {
        Self::new(
            function.get_block().clone(),
            function
                .iter_parameters()
                .map(Identifier::get_name)
                .map(ToOwned::to_owned)
                .collect(),
            function.is_variadic(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineFunction {
    implementation: EngineFunctionImpl,
    side_effects: bool,
}

impl EngineFunction {
    pub fn new(implementation: EngineFunctionImpl) -> Self {
        Self {
            implementation,
            side_effects: false,
        }
    }

    pub fn execute(&self, arguments: TupleValue) -> TupleValue {
        (self.implementation)(arguments)
    }

    pub fn has_side_effects(&self) -> bool {
        self.side_effects
    }
}
