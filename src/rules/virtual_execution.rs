use crate::nodes::{
    Block, FunctionExpression, FunctionStatement, Identifier, LocalFunctionStatement,
};
use crate::process::engine_impl::{
    create_roblox_bit32_library, create_roblox_math_library, create_roblox_string_library,
    create_tonumber, create_tostring, create_type,
};
use crate::process::{DefaultVisitor, LuaValue, NodeProcessor, NodeVisitor, VirtualLuaExecution};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::RulePropertyValue;

#[derive(Debug, Clone, Default)]
struct VirtualExecutionProcessor<F: Fn() -> VirtualLuaExecution> {
    build_executor: F,
}

impl<F: Fn() -> VirtualLuaExecution> VirtualExecutionProcessor<F> {
    fn new(build_executor: F) -> Self {
        Self { build_executor }
    }

    fn evaluate_block(&self, block: &mut Block) {
        let mut virtual_execution = (self.build_executor)();
        virtual_execution.evaluate_chunk(block);
    }
}

impl<F: Fn() -> VirtualLuaExecution> NodeProcessor for VirtualExecutionProcessor<F> {
    fn process_function_statement(&mut self, function: &mut FunctionStatement) {
        self.evaluate_block(function.mutate_block());
    }

    fn process_local_function_statement(&mut self, function: &mut LocalFunctionStatement) {
        self.evaluate_block(function.mutate_block());
    }

    fn process_function_expression(&mut self, function: &mut FunctionExpression) {
        self.evaluate_block(function.mutate_block());
    }
}

pub const VIRTUAL_EXECUTION_RULE_NAME: &str = "virtual_execution";

#[derive(Debug, PartialEq, Eq)]
struct EngineGlobal {
    identifier: &'static str,
    create_value: fn() -> LuaValue,
    property_name: &'static str,
}

/// A rule that runs Lua code as much as possible statically.
#[derive(Debug, PartialEq, Eq)]
pub struct VirtualExecution {
    globals: Vec<EngineGlobal>,
    throwaway_variable: String,
}

const DEFAULT_THROWAWAY_VARIABLE: &str = "_DARKLUA_THROWAWAY_VAR";

impl Default for VirtualExecution {
    fn default() -> Self {
        Self {
            globals: Default::default(),
            throwaway_variable: DEFAULT_THROWAWAY_VARIABLE.to_owned(),
        }
    }
}

impl VirtualExecution {
    fn build_virtual_lua_executor(&self) -> VirtualLuaExecution {
        self.globals.iter().fold(
            VirtualLuaExecution::default()
                .perform_mutations()
                .use_throwaway_variable(self.throwaway_variable.clone()),
            |execution, global| {
                execution.with_global_value(global.identifier, (global.create_value)())
            },
        )
    }

    fn include_globals(&mut self, list: Vec<String>) -> Result<(), RuleConfigurationError> {
        for value in list {
            match value.as_str() {
                "roblox-bit32" => {
                    self.globals.push(EngineGlobal {
                        identifier: "bit32",
                        create_value: create_roblox_bit32_library,
                        property_name: "roblox-bit32",
                    });
                }
                "roblox-math" => {
                    self.globals.push(EngineGlobal {
                        identifier: "math",
                        create_value: create_roblox_math_library,
                        property_name: "roblox-math",
                    });
                }
                "roblox-string" => {
                    self.globals.push(EngineGlobal {
                        identifier: "string",
                        create_value: create_roblox_string_library,
                        property_name: "roblox-string",
                    });
                }
                "tonumber" => {
                    self.globals.push(EngineGlobal {
                        identifier: "tonumber",
                        create_value: create_tonumber,
                        property_name: "tonumber",
                    });
                }
                "tostring" => {
                    self.globals.push(EngineGlobal {
                        identifier: "tostring",
                        create_value: create_tostring,
                        property_name: "tostring",
                    });
                }
                "type" => {
                    self.globals.push(EngineGlobal {
                        identifier: "type",
                        create_value: create_type,
                        property_name: "type",
                    });
                }
                _ => {
                    return Err(RuleConfigurationError::StringExpected(format!(
                        "invalid engine globals set `{}`",
                        value
                    )))
                }
            }
        }

        Ok(())
    }
}

impl FlawlessRule for VirtualExecution {
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut virtual_execution = self.build_virtual_lua_executor();
        virtual_execution.evaluate_chunk(block);

        let mut processor = VirtualExecutionProcessor::new(|| self.build_virtual_lua_executor());
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for VirtualExecution {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, value) in properties {
            match key.as_str() {
                "includes" => match value {
                    RulePropertyValue::StringList(includes) => self.include_globals(includes)?,
                    _ => return Err(RuleConfigurationError::StringListExpected(key)),
                },
                "throwaway_variable" => {
                    match value {
                        RulePropertyValue::String(throwaway_variable) => {
                            if Identifier::is_valid_identifier(&throwaway_variable) {
                                self.throwaway_variable = throwaway_variable;
                            } else {
                                return Err(RuleConfigurationError::UnexpectedValue {
                                property: key,
                                message: format!("variable `{}` is not valid, it must be a valid Lua identifier", throwaway_variable),
                            });
                            }
                        }
                        _ => return Err(RuleConfigurationError::StringExpected(key)),
                    }
                }
                _ => return Err(RuleConfigurationError::UnexpectedProperty(key)),
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        VIRTUAL_EXECUTION_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        let mut properties = RuleProperties::new();

        if !self.globals.is_empty() {
            properties.insert(
                "includes".to_owned(),
                self.globals
                    .iter()
                    .map(|global| global.property_name.to_owned())
                    .collect(),
            );
        }
        if self.throwaway_variable != DEFAULT_THROWAWAY_VARIABLE {
            properties.insert(
                "throwaway_variable".to_owned(),
                self.throwaway_variable.as_str().into(),
            );
        }

        properties
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    fn new_rule() -> VirtualExecution {
        VirtualExecution::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_virtual_execution", rule);
    }
}
