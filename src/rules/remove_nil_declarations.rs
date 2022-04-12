use crate::nodes::{Block, Expression, LocalAssignStatement};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

#[derive(Debug, Clone, Default)]
struct Processor {}

impl NodeProcessor for Processor {
    fn process_local_assign_statement(&mut self, assignment: &mut LocalAssignStatement) {
        while let Some(Expression::Nil(_)) = assignment.last_value() {
            assignment.pop_value();
        }
    }
}

pub const REMOVE_NIL_DECLARATION_RULE_NAME: &str = "remove_nil_declaration";

/// A rule that removes trailing `nil` in local assignments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveNilDeclaration {}

impl FlawlessRule for RemoveNilDeclaration {
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut processor = Processor::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveNilDeclaration {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_NIL_DECLARATION_RULE_NAME
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

    fn new_rule() -> RemoveNilDeclaration {
        RemoveNilDeclaration::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_nil_declaration", rule);
    }
}
