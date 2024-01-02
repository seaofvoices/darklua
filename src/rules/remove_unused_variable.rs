use crate::nodes::*;
use crate::process::processors::FindUsage;
use crate::process::{DefaultVisitor, Evaluator, NodeProcessor, NodeVisitor, ScopeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};
use crate::utils::expressions_as_statement;

use super::verify_no_rule_properties;

#[derive(Default)]
struct RemoveUnusedVariableProcessor {
    evaluator: Evaluator,
}

impl NodeProcessor for RemoveUnusedVariableProcessor {
    fn process_block(&mut self, block: &mut Block) {
        let length = block.statements_len();

        let assignments = block
            .reverse_iter_statements()
            .enumerate()
            .filter_map(|(i, statement)| match statement {
                Statement::LocalAssign(assignment) => {
                    let identifiers = assignment
                        .get_variables()
                        .iter()
                        .map(TypedIdentifier::get_identifier)
                        .map(Identifier::get_name)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>();

                    Some((length - i - 1, identifiers))
                }
                Statement::LocalFunction(function) => {
                    Some((length - i - 1, vec![function.get_name().to_owned()]))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let usages = assignments
            .into_iter()
            .map(|(index, identifiers)| {
                let usages = identifiers
                    .into_iter()
                    .map(|identifier| {
                        let mut find_usage = FindUsage::new(&identifier);

                        block
                            .iter_mut_statements()
                            .skip(index + 1)
                            .any(|next_statement| {
                                ScopeVisitor::visit_statement(next_statement, &mut find_usage);
                                find_usage.has_found_usage()
                            })
                            || block
                                .mutate_last_statement()
                                .into_iter()
                                .any(|last_statement| {
                                    ScopeVisitor::visit_last_statement(
                                        last_statement,
                                        &mut find_usage,
                                    );
                                    find_usage.has_found_usage()
                                })
                    })
                    .collect::<Vec<_>>();

                (index, usages)
            })
            .collect::<Vec<_>>();

        let mut usages_iter = usages.into_iter();

        let mut i = 0;
        let mut next_usage = usages_iter.next();

        block.filter_mut_statements(|statement| {
            let filter = if let Some((find_next_index, usages)) = &next_usage {
                let found = i == *find_next_index;
                let assign = if found {
                    if let Statement::LocalAssign(assign) = statement {
                        Some(assign)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let filter = if let Some(assign) = assign {
                    if usages.iter().all(|used| !used) {
                        let values = assign
                            .iter_values()
                            .filter(|value| self.evaluator.has_side_effects(value))
                            .cloned()
                            .collect::<Vec<_>>();

                        if values.is_empty() {
                            false
                        } else {
                            *statement = expressions_as_statement(values);
                            true
                        }
                    } else if usages.iter().any(|used| !used) {
                        let mut assignments: Vec<_> = assign
                            .iter_variables()
                            .zip(usages)
                            .map(|identifier| vec![identifier])
                            .zip(assign.iter_values())
                            .collect();

                        let length = assignments.len();
                        if let Some((last, value)) = assignments.last_mut() {
                            if self.evaluator.can_return_multiple_values(value) {
                                last.extend(assign.iter_variables().zip(usages).skip(length));
                            }
                        }

                        let mut variables = Vec::new();
                        let mut values = Vec::new();

                        for (mut identifiers, value) in assignments {
                            if !self.evaluator.has_side_effects(value) {
                                while identifiers.last().filter(|(_, used)| !**used).is_some() {
                                    identifiers.pop();
                                }
                            }

                            if !identifiers.is_empty() {
                                variables.extend(
                                    identifiers
                                        .into_iter()
                                        .map(|(identifier, _)| identifier.clone()),
                                );
                                values.push(value.clone());
                            }
                        }

                        if variables.is_empty() {
                            let extra_values: Vec<_> =
                                assign.iter_values().skip(length).cloned().collect();
                            if extra_values.is_empty() {
                                false
                            } else {
                                *statement = expressions_as_statement(extra_values);
                                true
                            }
                        } else {
                            values.extend(assign.iter_values().skip(length).cloned());
                            *statement = LocalAssignStatement::new(variables, values).into();
                            true
                        }
                    } else {
                        true
                    }
                } else {
                    usages.iter().any(|used| *used)
                };

                if found {
                    next_usage = usages_iter.next();
                }

                filter
            } else {
                true
            };

            i += 1;
            filter
        })
    }
}

pub const REMOVE_UNUSED_VARIABLE_RULE_NAME: &str = "remove_unused_variable";

/// A rule that removes unused variables.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveUnusedVariable {}

impl FlawlessRule for RemoveUnusedVariable {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = RemoveUnusedVariableProcessor::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveUnusedVariable {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_UNUSED_VARIABLE_RULE_NAME
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

    fn new_rule() -> RemoveUnusedVariable {
        RemoveUnusedVariable::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_unused_variable", rule);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'remove_unused_variable',
            prop: "something",
        }"#,
        );
        pretty_assertions::assert_eq!(result.unwrap_err().to_string(), "unexpected field 'prop'");
    }
}
