mod code_generator;
mod identifier_collector;
mod merge_functions;

use code_generator::*;
use identifier_collector::*;

use crate::nodes::Block;
use crate::process::{DefaultVisitor, NodeVisitor, ScopeVisitor};
use crate::rules::{
    Context,
    Rule,
    RuleConfiguration,
    RuleProcessResult,
    RuleConfigurationError,
    RuleProperties,
};

pub const CONVERT_LUX_TO_ROACT_CODE_RULE_NAME: &'static str = "convert_lux_to_roact_code";

/// A rule that removes LUX expressions and generates code for Roact.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConvertLUXToRoactCode {}

impl RuleConfiguration for ConvertLUXToRoactCode {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        // TODO: make the roact identifier configurable
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        CONVERT_LUX_TO_ROACT_CODE_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

impl Rule for ConvertLUXToRoactCode {
    fn process(&self, block: &mut Block, _context: &mut Context) -> RuleProcessResult {
        let mut identifiers = IdentifierCollector::default();
        DefaultVisitor::visit_block(block, &mut identifiers);

        let mut processor = RoactCodeGenerator::new(identifiers);
        ScopeVisitor::visit_block(block, &mut processor);

        processor.post_process_block(block);
        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     use insta::assert_json_snapshot;

//     fn new_rule() -> ConvertLUXToRoactCode {
//         ConvertLUXToRoactCode::default()
//     }

//     #[test]
//     fn serialize_default_rule() {
//         let rule: Box<dyn Rule> = Box::new(new_rule());

//         assert_json_snapshot!("default_convert_lux_to_roact_code", rule);
//     }
// }
