use crate::nodes::{
    self, BinaryExpression, Block, ElseIfExpressionBranch, Expression, IndexExpression,
    ParentheseExpression, TableEntry, TableExpression,
};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

#[derive(Default)]
struct Processor {}

fn process(
    condition: &Expression,
    result: &Expression,
    else_result: &Expression,
    branches: &Vec<&ElseIfExpressionBranch>,
) -> Expression {
    let result_parenthese = ParentheseExpression::new(result.clone());
    let result_table = TableExpression::new(vec![TableEntry::Value(result_parenthese.into())]);
    let if_bin = BinaryExpression::new(nodes::BinaryOperator::And, condition.clone(), result_table);
    let bin_right: Expression = if branches.len() > 0 {
        let b = branches[0];
        let elseif_result = process(
            b.get_condition(),
            b.get_result(),
            else_result,
            &branches[1..].to_vec(),
        );
        let elseif_result_table = TableExpression::new(vec![TableEntry::Value(elseif_result)]);
        elseif_result_table.into()
    } else {
        let else_result_parenthese = ParentheseExpression::new(else_result.clone());
        let else_result_table =
            TableExpression::new(vec![TableEntry::Value(else_result_parenthese.into())]);
        else_result_table.into()
    };
    let bin = BinaryExpression::new(nodes::BinaryOperator::Or, if_bin, bin_right);
    let bin_parenthese = ParentheseExpression::new(Expression::Binary(Box::new(bin)));
    IndexExpression::new(bin_parenthese, 1).into()
}

impl NodeProcessor for Processor {
    fn process_expression(&mut self, expression: &mut Expression) {
        if let Expression::If(if_exp) = expression {
            let translated_exp = process(
                if_exp.get_condition(),
                if_exp.get_result(),
                if_exp.get_else_result(),
                &if_exp.iter_branches().collect(),
            );
            *expression = translated_exp;
        }
    }
}

pub const REMOVE_IF_EXPRESSION_RULE_NAME: &str = "remove_if_expression";

/// A rule that removes trailing `nil` in local assignments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveIfExpression {}

impl FlawlessRule for RemoveIfExpression {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Processor::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveIfExpression {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_IF_EXPRESSION_RULE_NAME
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

    fn new_rule() -> RemoveIfExpression {
        RemoveIfExpression::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_if_expression", rule);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'remove_if_expression',
            prop: "something",
        }"#,
        );
        pretty_assertions::assert_eq!(result.unwrap_err().to_string(), "unexpected field 'prop'");
    }
}
