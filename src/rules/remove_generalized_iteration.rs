use crate::nodes::{Block, Expression, GenericForStatement, LocalAssignStatement};
use crate::process::{DefaultVisitor, Evaluator, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

#[derive(Default)]
struct Processor {}

impl NodeProcessor for Processor {
	fn process_generic_for_statement(&mut self, generic_for: &mut crate::nodes::GenericForStatement) {
		let iter_mut = generic_for.iter_mut_expressions();
		if iter_mut.count() > 1 {
			return;
		}
		for exp in iter_mut {

		}
	}
}

pub const REMOVE_GENERALIZED_ITERATION_RULE_NAME: &str = "remove_generalized_iteration";

/// A rule that removes trailing `nil` in local assignments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveGeneralizedIteration {}

impl FlawlessRule for RemoveGeneralizedIteration {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Processor::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveGeneralizedIteration {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_GENERALIZED_ITERATION_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rules::Rule;

    use insta::assert_json_snapshot;

    fn new_rule() -> RemoveGeneralizedIteration {
        RemoveGeneralizedIteration::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_generalized_iteration", rule);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'remove_generalized_iteration',
            prop: "something",
        }"#,
        );
        pretty_assertions::assert_eq!(result.unwrap_err().to_string(), "unexpected field 'prop'");
    }
}
