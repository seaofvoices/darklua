use crate::nodes::{Block, Expression};
use crate::process::{DefaultVisitorMut, Evaluator, NodeProcessorMut, NodeVisitorMut};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties};

#[derive(Debug, Clone, Default)]
struct Computer {
    evaluator: Evaluator,
}

impl Computer {
    fn replace_with(&self, expression: &mut Expression) -> Option<Expression> {
        match expression {
            Expression::Unary(_) | Expression::Binary(_) => {
                if !self.evaluator.has_side_effects(&expression) {
                    self.evaluator.evaluate(&expression)
                        .to_expression()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl NodeProcessorMut for Computer {
    fn process_expression(&mut self, expression: &mut Expression) {
        if let Some(replace_with) = self.replace_with(expression) {
            *expression = replace_with;
        }
    }
}

pub const COMPUTE_EXPRESSIONS_RULE_NAME: &'static str = "compute_expression";

/// A rule that compute expressions that do not have any side-effects.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ComputeExpression {}

impl Rule for ComputeExpression {
    fn process(&self, block: &mut Block) {
        let mut processor = Computer::default();
        DefaultVisitorMut::visit_block(block, &mut processor);
    }

    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        COMPUTE_EXPRESSIONS_RULE_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_json_snapshot;

    fn new_rule() -> ComputeExpression {
        ComputeExpression::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_compute_expression", rule);
    }
}
