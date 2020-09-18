use crate::nodes::{Block, Statement};
use crate::process::{DefaultVisitorMut, Evaluator, NodeProcessorMut, NodeVisitorMut};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties};

#[derive(Debug, Clone, Default)]
struct WhileFilter {
    evaluator: Evaluator,
}

impl NodeProcessorMut for WhileFilter {
    fn process_block(&mut self, block: &mut Block) {
        block.filter_statements(|statement| match statement {
            Statement::While(while_statement) => {
                let condition = while_statement.get_condition();

                self.evaluator.has_side_effects(condition)
                || self.evaluator.evaluate(condition).is_truthy().unwrap_or(true)
            },
            _ => true,
        });
    }
}

pub const REMOVE_UNUSED_WHILE_RULE_NAME: &'static str = "remove_unused_while";

/// A rule that removes while statements with a known false condition.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveUnusedWhile {}

impl Rule for RemoveUnusedWhile {
    fn process(&self, block: &mut Block) {
        let mut processor = WhileFilter::default();
        DefaultVisitorMut::visit_block(block, &mut processor);
    }

    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_UNUSED_WHILE_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_json_snapshot;

    fn new_rule() -> RemoveUnusedWhile {
        RemoveUnusedWhile::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_unused_while", rule);
    }
}
