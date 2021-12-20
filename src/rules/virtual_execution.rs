use crate::nodes::Block;
use crate::process::engine_impl::{
    create_roblox_math_library, create_tonumber, create_tostring, create_type, create_roblox_string_library,
};
use crate::process::{LuaValue, VirtualLuaExecution};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::RulePropertyValue;

pub const VIRTUAL_EXECUTION_RULE_NAME: &str = "virtual_execution";

#[derive(Debug, PartialEq, Eq)]
struct EngineGlobal {
    identifier: &'static str,
    create_value: fn() -> LuaValue,
    property_name: &'static str,
}

/// A rule that runs Lua code as much as possible statically.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct VirtualExecution {
    globals: Vec<EngineGlobal>,
}

impl VirtualExecution {
    fn include_globals(&mut self, list: Vec<String>) -> Result<(), RuleConfigurationError> {
        for value in list {
            match value.as_str() {
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
        let mut virtual_execution = self.globals.iter().fold(
            VirtualLuaExecution::default().perform_mutations(),
            |execution, global| {
                execution.with_global_value(global.identifier, (global.create_value)())
            },
        );

        virtual_execution.evaluate_chunk(block);
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
