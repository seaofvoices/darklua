mod find_variable;
mod refactor_assignments;
mod remove_variables;

use find_variable::FindVariable;
use refactor_assignments::refactor_assignments;
use remove_variables::RemoveVariables;

use crate::nodes::{
    Block,
    DoStatement,
    LastStatement,
    LocalAssignStatement,
    Statement,
};
use crate::process::{
    DefaultVisitorMut,
    Evaluator,
    NodeProcessorMut,
    NodeVisitor,
    NodeVisitorMut,
    ScopeVisitor,
};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties};

#[derive(Debug, Clone, PartialEq, Eq)]
enum FilterResult {
    Keep,
    Remove,
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum Modification {
    Remove(Vec<String>),
    Replace(Statement, Vec<String>),
}

#[derive(Debug, Clone, Default)]
struct UnusedFilter {
    evaluator: Evaluator,
}

impl UnusedFilter {
    fn find_usage<'a, I: Iterator<Item=&'a Statement>>(
        &self,
        find_variable: &mut FindVariable,
        mut next_statements: I,
        last_statement: &Option<LastStatement>
    ) -> FilterResult {
        next_statements
            .find(|statement| {
                ScopeVisitor::visit_statement(statement, find_variable);

                find_variable.has_found_usage()
            })
            .map(|_| FilterResult::Keep)
            .or_else(|| {
                last_statement.as_ref()
                    .filter(|last_statement| {
                        ScopeVisitor::visit_last_statement(last_statement, find_variable);

                        find_variable.has_found_usage()
                    })
                    .map(|_| FilterResult::Keep)
            })
            .unwrap_or(FilterResult::Remove)
    }
}

impl NodeProcessorMut for UnusedFilter {
    fn process_block(&mut self, block: &mut Block) {
        let (statements, last_statement) = block.mutate_all_statements();

        for i in (0..statements.len()).rev() {
            if let Some(modification) = {
                let current = statements.get(i).unwrap();
                let next_statements = statements.iter().skip(i + 1);

                match current {
                    Statement::LocalFunction(function) => {
                        let mut find_function = FindVariable::new(function.get_identifier());

                        match self.find_usage(&mut find_function, next_statements, last_statement) {
                            FilterResult::Remove => {
                                Some(Modification::Remove(vec![
                                    function.get_identifier().to_owned(),
                                ]))
                            }
                            _ => None,
                        }
                    }
                    Statement::LocalAssign(assign) => {
                        let values = assign.get_values();
                        let assignments: Vec<(String, bool)> = assign
                            .get_variables()
                            .iter()
                            .map(|identifier| {
                                let mut find_identifier = FindVariable::new(identifier);

                                let usage = self.find_usage(
                                    &mut find_identifier,
                                    next_statements.clone(),
                                    last_statement,
                                );

                                (identifier.clone(), usage == FilterResult::Keep)
                            })
                            .collect();
                        let removed_variable_names = assignments.iter()
                            .filter_map(|(identifier, keep)| {
                                if !keep {
                                    Some(identifier.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        let mut refactor = refactor_assignments(
                            assignments,
                            values.into_iter().collect(),
                            &self.evaluator,
                        );

                        if refactor.variables.is_empty() {
                            if refactor.statements.is_empty() {
                                Some(Modification::Remove(removed_variable_names))
                            } else {
                                if refactor.statements.len() == 1 {
                                    Some(Modification::Replace(
                                        refactor.statements.pop().unwrap(),
                                        removed_variable_names,
                                    ))
                                } else {
                                    Some(Modification::Replace(
                                        DoStatement::new(
                                            Block::new(refactor.statements, None)
                                        ).into(),
                                        removed_variable_names,
                                    ))
                                }
                            }
                        } else {
                            Some(Modification::Replace(
                                LocalAssignStatement::new(
                                    refactor.variables,
                                    refactor.values,
                                ).into(),
                                removed_variable_names,
                            ))
                        }
                    }
                    _ => None,
                }
            } {
                let (variables, remove_from) = match modification {
                    Modification::Remove(variables) => {
                        statements.remove(i);
                        (variables, i)
                    }
                    Modification::Replace(new_statement, variables) => {
                        *statements.get_mut(i).unwrap() = new_statement;
                        (variables, i + 1)
                    }
                };
                println!("remove variables {:?}", variables);

                if variables.is_empty() {
                    continue
                }

                for i in (remove_from..statements.len()).rev() {
                    if let Some(statement) = statements.get_mut(i) {
                        println!("test statement {:?}", statement);
                        let mut remove_variables = RemoveVariables::new(variables.clone());
                        DefaultVisitorMut::visit_statement(statement, &mut remove_variables);

                        let remove_statement = match statement {
                            Statement::Do(do_statement) => do_statement.is_empty(),
                            _ => false,
                        };

                        if remove_statement {
                            statements.remove(i);
                        }
                    } else {
                        break
                    }
                }
            }
        }
    }
}

pub const REMOVE_UNUSED_VARIABLE_RULE_NAME: &'static str = "remove_unused_variable";

/// A rule that removes unused variables.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveUnusedVariable {}

impl Rule for RemoveUnusedVariable {
    fn process(&self, block: &mut Block) {
        let mut processor = UnusedFilter::default();
        DefaultVisitorMut::visit_block(block, &mut processor);
    }

    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

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

    use insta::assert_json_snapshot;

    fn new_rule() -> RemoveUnusedVariable {
        RemoveUnusedVariable::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_unused_variable", rule);
    }
}
