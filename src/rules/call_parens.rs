use crate::nodes::{Arguments, Block, Expression, FunctionCall, StringExpression, TableExpression};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use std::mem;

#[derive(Debug, Clone, Default)]
struct Processor {}

impl NodeProcessor for Processor {
    fn process_function_call(&mut self, call: &mut FunctionCall) {
        let new_arguments = match call.mutate_arguments() {
            Arguments::Tuple(expressions) if expressions.len() == 1 => {
                let expression = expressions.iter_mut().next().unwrap();

                match expression {
                    Expression::String(string) => {
                        let mut steal_string = StringExpression::empty();
                        mem::swap(string, &mut steal_string);
                        Some(Arguments::String(steal_string))
                    }
                    Expression::Table(table) => {
                        let mut steal_table = TableExpression::default();
                        mem::swap(table, &mut steal_table);
                        Some(Arguments::Table(steal_table))
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        if let Some(new_arguments) = new_arguments {
            *call.mutate_arguments() = new_arguments;
        }
    }
}

pub const REMOVE_FUNCTION_CALL_PARENS: &'static str = "remove_function_call_parens";

/// A rule that removes parentheses when calling functions with a string or a table.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveFunctionCallParens {}

impl FlawlessRule for RemoveFunctionCallParens {
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut processor = Processor::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveFunctionCallParens {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key));
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_FUNCTION_CALL_PARENS
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

    fn new_rule() -> RemoveFunctionCallParens {
        RemoveFunctionCallParens::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_function_call_parens", rule);
    }
}
