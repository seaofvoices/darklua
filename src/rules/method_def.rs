use crate::nodes::{Block, FunctionStatement};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties};

#[derive(Default)]
struct FunctionMutator;

impl NodeProcessor for FunctionMutator {
    fn process_function_statement(&mut self, function: &mut FunctionStatement) {
        function.remove_method();
    }
}

pub const REMOVE_METHOD_DEFINITION_RULE_NAME: &'static str = "remove_method_definition";

/// Change method functions into regular functions.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveMethodDefinition {}

impl Rule for RemoveMethodDefinition {
    fn process(&self, block: &mut Block) {
        let mut processor = FunctionMutator::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }

    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_METHOD_DEFINITION_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_json_snapshot;

    fn new_rule() -> RemoveMethodDefinition {
        RemoveMethodDefinition::default()
    }

    fn wrap(rule: RemoveMethodDefinition) -> Box<dyn Rule> {
        Box::new(rule)
    }

    #[test]
    fn serialize_default_rule() {
        assert_json_snapshot!("default_remove_method_definition", wrap(new_rule()));
    }
}
