use crate::nodes::Block;

use super::LuaValue;

pub type EngineFunctionImpl = fn(Vec<LuaValue>) -> Vec<LuaValue>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionValue {
    Lua(LuaFunction),
    Engine(EngineFunction),
}

impl FunctionValue {
    pub fn execute(&self, arguments: Vec<LuaValue>) -> Vec<LuaValue> {
        match self {
            FunctionValue::Lua(_) => todo!(),
            FunctionValue::Engine(engine) => engine.execute(arguments),
        }
    }
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineFunction {
    implementation: EngineFunctionImpl,
}

impl EngineFunction {
    pub fn new(implementation: EngineFunctionImpl) -> Self {
        Self { implementation }
    }

    fn execute(&self, arguments: Vec<LuaValue>) -> Vec<LuaValue> {
        (self.implementation)(arguments)
    }
}
