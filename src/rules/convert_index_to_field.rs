use crate::nodes::{
    Block, Expression, FieldExpression, Identifier, IndexExpression, Prefix, Variable,
};
use crate::process::{
    utils::keywords, DefaultVisitor, Evaluator, LuaValue, NodeProcessor, NodeVisitor,
};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

use std::mem;

#[derive(Debug, Clone, Default)]
struct Converter {
    evaluator: Evaluator,
}

fn is_identifier(string: &str) -> bool {
    string.starts_with(|c: char| c.is_ascii_alphabetic())
        && string.chars().skip(1).all(|c| c.is_ascii_alphanumeric())
        && !matches!(string, keywords::matches_any!())
}

impl Converter {
    fn convert_to_field(&self, index: &IndexExpression) -> Option<FieldExpression> {
        if let LuaValue::String(string) = self.evaluator.evaluate(index.get_index()) {
            if is_identifier(&string) {
                return Some(FieldExpression::new(
                    index.get_prefix().clone(),
                    Identifier::new(string),
                ));
            }
        }
        None
    }
}

impl NodeProcessor for Converter {
    fn process_expression(&mut self, expression: &mut Expression) {
        let field: Option<Expression> = if let Expression::Index(index) = expression {
            self.convert_to_field(index).map(Into::into)
        } else {
            None
        };
        if let Some(mut field) = field {
            mem::swap(expression, &mut field);
        }
    }

    fn process_prefix_expression(&mut self, prefix: &mut Prefix) {
        let field: Option<Prefix> = if let Prefix::Index(index) = prefix {
            self.convert_to_field(index).map(Into::into)
        } else {
            None
        };
        if let Some(mut field) = field {
            mem::swap(prefix, &mut field);
        }
    }

    fn process_variable(&mut self, variable: &mut Variable) {
        let field: Option<Variable> = if let Variable::Index(index) = variable {
            self.convert_to_field(index).map(Into::into)
        } else {
            None
        };
        if let Some(mut field) = field {
            mem::swap(variable, &mut field);
        }
    }
}

pub const CONVERT_INDEX_TO_FIELD_RULE_NAME: &str = "convert_index_to_field";

/// A rule that converts index expression into field expression.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConvertIndexToField {}

impl FlawlessRule for ConvertIndexToField {
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut processor = Converter::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for ConvertIndexToField {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        CONVERT_INDEX_TO_FIELD_RULE_NAME
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

    fn new_rule() -> ConvertIndexToField {
        ConvertIndexToField::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_convert_index_to_field", rule);
    }
}
