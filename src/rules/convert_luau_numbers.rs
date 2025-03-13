use crate::{
    nodes::{Block, DecimalNumber, NumberExpression},
    process::{DefaultVisitor, NodeProcessor, NodeVisitor},
    rules::{Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties},
};

use super::verify_no_rule_properties;

#[derive(Default)]
struct Processor<'a> {
    code: &'a str,
}

impl NodeProcessor for Processor<'_> {
    fn process_number_expression(&mut self, num_exp: &mut NumberExpression) {
        if let NumberExpression::Binary(binary) = num_exp {
            let value = binary.compute_value();
            *num_exp = DecimalNumber::new(value).into();
            return;
        }
        if let Some(token) = num_exp.get_token() {
            let content = token.read(self.code);
            let mut underscore_removed = String::with_capacity(content.len());
            let mut changed = false;

            for c in content.chars() {
                if c != '_' {
                    underscore_removed.push(c);
                } else {
                    changed = true;
                }
            }

            if changed {
                let mut new_token = token.clone();
                new_token.replace_with_content(underscore_removed);
                num_exp.set_token(new_token);
            }
        }
    }
}

impl<'a> Processor<'a> {
    fn new(code: &'a str) -> Self {
        Self { code }
    }
}

pub const CONVERT_LUAU_NUMBERS_RULE_NAME: &str = "convert_luau_numbers";

/// A rule that converts Luau number literals to decimal numbers.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct ConvertLuauNumbers {}

impl FlawlessRule for ConvertLuauNumbers {
    fn flawless_process(&self, block: &mut Block, context: &Context) {
        let mut processor = Processor::new(context.original_code());
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for ConvertLuauNumbers {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        CONVERT_LUAU_NUMBERS_RULE_NAME
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

    fn new_rule() -> ConvertLuauNumbers {
        ConvertLuauNumbers::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_convert_luau_numbers", rule);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'convert_luau_numbers',
            prop: "something",
        }"#,
        );
        pretty_assertions::assert_eq!(result.unwrap_err().to_string(), "unexpected field 'prop'");
    }
}
