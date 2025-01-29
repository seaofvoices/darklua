use crate::{
    nodes::{Block, DecimalNumber, Expression, NumberExpression},
    process::{DefaultVisitor, NodeProcessor, NodeVisitor},
    rules::{Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties},
};

use super::verify_no_rule_properties;

#[derive(Default)]
struct Processor {}

impl NodeProcessor for Processor {
    fn process_expression(&mut self, exp: &mut Expression) {
        if let Expression::Number(num_exp) = exp {
            match num_exp {
                NumberExpression::Binary(binary) => {
                    let value = binary.compute_value();
                    *exp = DecimalNumber::new(value).into();
                }
                NumberExpression::Decimal(decimal) => {
                    let value = decimal.compute_value();
                    *exp = DecimalNumber::new(value).into();
                }
                NumberExpression::Hex(hex) => {
                    let value = hex.compute_value();
                    *exp = DecimalNumber::new(value).into();
                }
            }
        }
    }
}

pub const REMOVE_NUMBER_LITERALS_RULE_NAME: &str = "remove_number_literals";

/// A rule that removes number literals.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct RemoveNumberLiterals {}

impl FlawlessRule for RemoveNumberLiterals {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Processor {};
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveNumberLiterals {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_NUMBER_LITERALS_RULE_NAME
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

    fn new_rule() -> RemoveNumberLiterals {
        RemoveNumberLiterals::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_number_literals", rule);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'remove_number_literals',
            prop: "something",
        }"#,
        );
        pretty_assertions::assert_eq!(result.unwrap_err().to_string(), "unexpected field 'prop'");
    }
}
