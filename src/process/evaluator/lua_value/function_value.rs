use crate::nodes::Block;

use super::TupleValue;

pub type EngineFunctionImpl = fn(TupleValue) -> TupleValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionValue {
    Lua(LuaFunction),
    Engine(EngineFunction),
}

impl FunctionValue {
    pub fn execute(&self, arguments: TupleValue) -> TupleValue {
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
