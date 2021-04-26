use crate::nodes::{Block, LocalAssignStatement, Expression, Statement};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::process::processors::FindVariables;
use crate::rules::{Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties};

use std::iter;

#[derive(Debug, Clone, Default)]
struct GroupLocalProcessor {}

impl GroupLocalProcessor {
    fn filter_statements(&self, block: &mut Block) -> Vec<Statement> {
        let statements = block.mutate_statements();
        let mut filter_statements = Vec::new();
        let mut iter = statements.drain(..);
        let mut previous_statement = iter.next();
        let mut current_statement = iter.next();

        while let Some(current) = current_statement {
            previous_statement = if let Some(previous) = previous_statement {
                use Statement::LocalAssign;

                match (previous, current) {
                    (LocalAssign(mut previous), LocalAssign(mut current)) => {
                        if self.should_merge(&previous, &mut current) {
                            self.merge(&mut previous, current);

                            Some(LocalAssign(previous))
                        } else {
                            filter_statements.push(LocalAssign(previous));
                            Some(LocalAssign(current))
                        }
                    }
                    (previous, current) => {
                        filter_statements.push(previous);
                        Some(current)
                    },
                }
            } else {
                None
            };

            current_statement = iter.next();
        }

        if let Some(previous) = previous_statement {
            filter_statements.push(previous);
        }

        filter_statements
    }

    fn should_merge(&self, first: &LocalAssignStatement, next: &mut LocalAssignStatement) -> bool {
        let first_value_count = first.value_count();

        if first.variable_count() > first_value_count && first_value_count != 0 {
            return false
        }

        let mut find_variables = FindVariables::from(first.get_variables());

        next.mutate_values().iter_mut()
            .all(|expression| {
                DefaultVisitor::visit_expression(expression, &mut find_variables);
                !find_variables.has_found_usage()
            })
    }

    fn merge(&self, first: &mut LocalAssignStatement, mut other: LocalAssignStatement) {
        if first.value_count() == 0 && other.value_count() != 0 {
            let variable_count = first.variable_count();
            first.mutate_values()
                .extend(iter::repeat(Expression::Nil).take(variable_count));
        }

        if other.value_count() == 0 && first.value_count() != 0 {
            let variable_count = other.variable_count();
            other.mutate_values()
                .extend(iter::repeat(Expression::Nil).take(variable_count));
        }

        first.mutate_variables()
            .extend(other.mutate_variables().drain(..));
        first.mutate_values()
            .extend(other.mutate_values().drain(..));
    }
}

impl NodeProcessor for GroupLocalProcessor {
    fn process_block(&mut self, block: &mut Block) {
        let filter_statements = self.filter_statements(block);

        *block.mutate_statements() = filter_statements;
    }
}

pub const GROUP_LOCAL_ASSIGNMENT: &'static str = "group_local_assignment";

/// Group local assign statements into one statement.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct GroupLocalAssignment {}

impl FlawlessRule for GroupLocalAssignment {
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut processor = GroupLocalProcessor::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for GroupLocalAssignment {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        GROUP_LOCAL_ASSIGNMENT
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

    fn new_rule() -> GroupLocalAssignment {
        GroupLocalAssignment::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_group_local_assignment", rule);
    }
}
