use crate::nodes::{
    Block,
    DoStatement,
    Expression,
    LastStatement,
    LocalAssignStatement,
    LocalFunctionStatement,
    Statement,
};
use crate::process::{
    DefaultVisitorMut,
    Evaluator,
    NodeProcessor,
    NodeProcessorMut,
    NodeVisitor,
    NodeVisitorMut,
    Scope,
    ScopeVisitor,
};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties};

use std::collections::HashSet;
use std::mem;

#[derive(Debug, Clone)]
struct FindVariable {
    usage_found: bool,
    identifier: String,
    identifiers: Vec<HashSet<String>>,
}

impl FindVariable {
    pub fn new<S: Into<String>>(identifier: S) -> Self {
        Self {
            usage_found: false,
            identifier: identifier.into(),
            identifiers: Vec::new(),
        }
    }

    #[inline]
    pub fn has_found_usage(&self) -> bool {
        self.usage_found
    }

    fn is_identifier_used(&self, identifier: &String) -> bool {
        self.identifiers.iter()
            .any(|set| set.contains(identifier))
    }

    fn insert_identifier(&mut self, identifier: &String) {
        if let Some(set) = self.identifiers.last_mut() {
            set.insert(identifier.clone());
        } else {
            let mut set = HashSet::new();
            set.insert(identifier.clone());
            self.identifiers.push(set);
        }
    }
}

impl NodeProcessor for FindVariable {
    fn process_variable_expression(&mut self, variable: &String) {
        if !self.usage_found {
            if variable == &self.identifier && !self.is_identifier_used(variable) {
                self.usage_found = true;
            }
        }
    }
}

impl Scope for FindVariable {
    fn push(&mut self) {
        self.identifiers.push(HashSet::new())
    }

    fn pop(&mut self) {
        self.identifiers.pop();
    }

    fn insert(&mut self, identifier: &String) {
        self.insert_identifier(identifier);
    }

    fn insert_local(&mut self, identifier: &String, _value: Option<&Expression>) {
        self.insert_identifier(identifier);
    }

    fn insert_local_function(&mut self, function: &LocalFunctionStatement) {
        self.insert_identifier(function.get_identifier());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FilterResult {
    Keep,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Modification {
    Remove,
    Replace(Statement),
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
                                Some(Modification::Remove)
                            }
                            _ => None,
                        }
                    }
                    Statement::LocalAssign(assign) => {
                        let usage: Vec<FilterResult> = assign.get_variables().iter()
                            .map(|identifier| {
                                let mut find_identifier = FindVariable::new(identifier);

                                self.find_usage(&mut find_identifier, next_statements.clone(), last_statement)
                            })
                            .collect();

                        let side_effects: Vec<bool> = assign.get_values().iter()
                            .map(|value| {
                                self.evaluator.has_side_effects(value)
                            })
                            .collect();

                        if usage.iter().all(|result| result == &FilterResult::Remove) {
                            if side_effects.iter().all(|effect| !effect) {
                                Some(Modification::Remove)
                            } else {
                                let mut calls: Vec<Statement> = assign.get_values().iter()
                                    .filter_map(|value| {
                                        match value {
                                            Expression::Call(call) => {
                                                Some(Statement::Call(*call.clone()))
                                            }
                                            _ => None,
                                        }
                                    })
                                    .collect();
                                let values: Vec<Expression> = assign.get_values().iter()
                                    .filter_map(|value| {
                                        match value {
                                            Expression::Call(_) => None,
                                            _ => Some(value.clone()),
                                        }
                                    })
                                    .collect();

                                match (calls.len(), values.len()) {
                                    (0, 0) => Some(Modification::Remove),
                                    (1, 0) => {
                                        Some(Modification::Replace(calls.pop().unwrap()))
                                    }
                                    (0, _) => {
                                        let identifier = assign.get_variables()
                                            .first()
                                            .cloned()
                                            .unwrap();
                                        let statement = LocalAssignStatement::new(
                                            vec![identifier],
                                            values,
                                        );

                                        Some(Modification::Replace(statement.into()))
                                    }
                                    (_, 0) => {
                                        let statement = DoStatement::new(Block::new(calls, None));
                                        Some(Modification::Replace(statement.into()))
                                    }
                                    _ => {
                                        calls.push(LocalAssignStatement::new(
                                            vec!["_".to_owned()],
                                            values,
                                        ).into());
                                        let statement = DoStatement::new(Block::new(calls, None));
                                        Some(Modification::Replace(statement.into()))
                                    }
                                }
                            }
                        } else {
                            let value_count = assign.value_count();
                            let variable_count = assign.variable_count();

                            let mut values = assign.get_values().iter()
                                .zip(side_effects.iter())
                                .rev();
                            let values = values.by_ref();

                            let extra_value_count = if value_count >= variable_count {
                                value_count - variable_count
                            } else {
                                0
                            };
                            let extra_values: Vec<Expression> = values.take(extra_value_count)
                                .filter(|(_value, side_effect)| **side_effect)
                                .map(|(value, _)| value.clone())
                                .collect();

                            let mut variables = assign.get_variables().iter()
                                .zip(usage.iter())
                                .rev();
                            let variables = variables.by_ref();

                            let last_variables_count = 1 + if variable_count >= value_count {
                                variable_count - value_count
                            } else {
                                0
                            };
                            let last_variables_for_last_value = variables
                                .take(last_variables_count);

                            let mut reversed_values_for_assignment = Vec::new();
                            let mut reversed_identifiers: Vec<String> = last_variables_for_last_value
                                .skip_while(|(_, usage)| **usage == FilterResult::Remove)
                                .map(|(identifier, _)| identifier.to_owned())
                                .collect();

                            if !reversed_identifiers.is_empty() {
                                if let Some((last_value, _side_effect)) = values.next() {
                                    reversed_values_for_assignment.push(last_value.clone());
                                }
                            }

                            while let Some((value, has_side_effect)) = values.next() {
                                let (identifier, usage) = variables.next()
                                    .expect("found lower than expected amount of variables");

                                if *usage == FilterResult::Keep || *has_side_effect {
                                    reversed_identifiers.push(identifier.to_string());
                                    reversed_values_for_assignment.push(value.clone());
                                }
                            }

                            reversed_identifiers.reverse();
                            reversed_values_for_assignment.reverse();
                            reversed_values_for_assignment.extend(extra_values.into_iter());

                            let statement = LocalAssignStatement::new(
                                reversed_identifiers,
                                reversed_values_for_assignment,
                            );

                            Some(Modification::Replace(statement.into()))

                        }
                    }
                    _ => None,
                }
            } {
                match modification {
                    Modification::Remove => {
                        statements.remove(i);
                    }
                    Modification::Replace(new_statement) => {
                        mem::replace(statements.get_mut(i).unwrap(), new_statement);
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
