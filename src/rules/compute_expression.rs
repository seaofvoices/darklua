use crate::nodes::{Block, Expression};
use crate::process::{DefaultVisitor, Evaluator, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

#[derive(Debug, Clone, Default)]
struct Computer {
    evaluator: Evaluator,
}

impl Computer {
    fn replace_with(&self, expression: &mut Expression) -> Option<Expression> {
        match expression {
            Expression::Unary(_) | Expression::Binary(_) => {
                if !self.evaluator.has_side_effects(&expression) {
                    self.evaluator.evaluate(&expression).to_expression()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl NodeProcessor for Computer {
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

impl FlawlessRule for ComputeExpression {
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut processor = Computer::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for ComputeExpression {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key));
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
    use crate::rules::Rule;

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
