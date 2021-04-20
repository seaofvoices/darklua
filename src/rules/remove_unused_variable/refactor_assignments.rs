use crate::nodes::{
    Expression,
    Statement,
};
use crate::process::Evaluator;

#[derive(Debug, PartialEq, Eq)]
pub struct RefactorResult<T> {
    pub statements: Vec<Statement>,
    pub variables: Vec<T>,
    pub values: Vec<Expression>,
}

impl<T> RefactorResult<T> {
    fn empty() -> Self {
        Self {
            statements: Vec::new(),
            variables: Vec::new(),
            values: Vec::new(),
        }
    }

    fn with_statements(mut self, statements: Vec<Statement>) -> Self {
        self.statements = statements;
        self
    }

    fn with_variables(mut self, variables: Vec<T>) -> Self {
        self.variables = variables;
        self
    }

    fn with_values(mut self, values: Vec<Expression>) -> Self {
        self.values = values;
        self
    }
}

fn refactor_remove_all<T>(
    // a vector of variables and a boolean that tells if that variable
    // should be kept and if it contains side effects
    assignments: Vec<(T, bool, bool)>,
    values: Vec<&Expression>,
    evaluator: &Evaluator,
) -> RefactorResult<T> {
    let keep_values: Vec<&Expression> = values.into_iter()
        .filter(|value| evaluator.has_side_effects(value))
        .collect();

    let mut calls_at_front = Vec::new();
    let mut remaining_values = Vec::new();
    let mut only_function_calls = true;

    keep_values.into_iter()
        .for_each(|value| {
            if only_function_calls {
                match value {
                    Expression::Call(call) => {
                        calls_at_front.push(
                            Statement::Call(call.as_ref().clone())
                        );
                    }
                    _ => {
                        only_function_calls = false;
                        remaining_values.push(value.clone());
                    }
                }
            } else {
                remaining_values.push(value.clone());
            }
        });

    if remaining_values.is_empty() {
        RefactorResult::empty()
            .with_statements(calls_at_front)
    } else {
        let (variable, _, _) = assignments.into_iter()
            .next()
            .unwrap();
        RefactorResult::empty()
            .with_statements(calls_at_front)
            .with_variables(vec![variable])
            .with_values(remaining_values)
    }
}

pub fn refactor_assignments<T>(
    // a vector of variables and a boolean that tells if that variable
    // should be kept and if it contains side effects
    assignments: Vec<(T, bool, bool)>,
    values: Vec<&Expression>,
    evaluator: &Evaluator,
) -> RefactorResult<T> {
    if assignments.iter().all(|(_, keep, _side_effect)| !keep) {
        refactor_remove_all(assignments, values, evaluator)
    } else {
        let value_count = values.len();
        let variable_count = assignments.len();

        let mut reversed_values = values.into_iter().rev();
        let reversed_values = reversed_values.by_ref();

        let extra_value_count = if value_count >= variable_count {
            value_count - variable_count
        } else {
            0
        };

        let extra_values: Vec<Expression> = reversed_values.take(extra_value_count)
            .filter_map(|value| {
                if evaluator.has_side_effects(value) {
                    Some((*value).clone())
                } else {
                    None
                }
            })
            .collect();

        let mut variables = assignments.into_iter().rev();

        let variables = variables.by_ref();

        let last_variables_count = 1 + if variable_count >= value_count {
            variable_count - value_count
        } else {
            0
        };
        let last_variables_for_last_value = variables.take(last_variables_count);

        let mut reversed_values_for_assignment = Vec::new();
        let mut reversed_variables: Vec<T> = last_variables_for_last_value
            .skip_while(|(_, keep, _)| !*keep)
            .map(|(variable, _, _)| variable)
            .collect();

        if !reversed_variables.is_empty() {
            if let Some(last_value) = reversed_values.next() {
                reversed_values_for_assignment.push(last_value.clone());
            }
        }

        while let Some(value) = reversed_values.next() {
            let (variable, keep_variable, _has_side_effect) = variables.next()
                .expect("found lower than expected amount of variables");

            if keep_variable || evaluator.has_side_effects(value) {
                reversed_variables.push(variable);
                reversed_values_for_assignment.push(value.clone());
            }
        }

        reversed_variables.reverse();
        reversed_values_for_assignment.reverse();
        reversed_values_for_assignment.extend(extra_values.into_iter());

        RefactorResult {
            statements: Vec::new(),
            variables: reversed_variables,
            values: reversed_values_for_assignment,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nodes::FunctionCall;

    #[test]
    fn removes_variable_without_a_value() {
        let evaluator = Evaluator::default();
        let refactor = refactor_assignments(
            vec![("foo", false, false)],
            Vec::new(),
            &evaluator,
        );
        assert_eq!(refactor, RefactorResult::empty());
    }

    #[test]
    fn keeps_variable_if_marked_as_used() {
        let evaluator = Evaluator::default();
        let refactor = refactor_assignments(
            vec![("foo", true, false)],
            Vec::new(),
            &evaluator,
        );
        let expected = RefactorResult::empty()
            .with_variables(vec!["foo"]);
        assert_eq!(refactor, expected);
    }

    #[test]
    fn returns_a_call_statement() {
        let evaluator = Evaluator::default();
        let call = FunctionCall::from_name("print");
        let call_expression = call.clone().into();
        let refactor = refactor_assignments(
            vec![("foo", false, false)],
            vec![&call_expression],
            &evaluator,
        );
        let expected = RefactorResult::empty()
            .with_statements(vec![call.into()]);
        assert_eq!(refactor, expected);
    }

    #[test]
    fn keeps_unused_variable_for_tuple_extraction() {
        let evaluator = Evaluator::default();
        let refactor = refactor_assignments(
            vec![
                ("a", false, false),
                ("b", false, false),
                ("c", true, false),
            ],
            vec![&Expression::True, &Expression::VariableArguments],
            &evaluator,
        );
        let expected = RefactorResult::empty()
            .with_variables(vec!["b", "c"])
            .with_values(vec![Expression::VariableArguments]);
        assert_eq!(refactor, expected);
    }

    #[test]
    fn keeps_extra_values_after_used_variable() {
        let evaluator = Evaluator::default();
        let call = FunctionCall::from_name("print");
        let call_expression = call.clone().into();
        let refactor = refactor_assignments(
            vec![("a", true, false)],
            vec![&Expression::True, &call_expression],
            &evaluator,
        );
        let expected = RefactorResult::empty()
            .with_variables(vec!["a"])
            .with_values(vec![Expression::True, call_expression]);
        assert_eq!(refactor, expected);
    }

    #[test]
    fn keeps_uninitialized_variable() {
        let evaluator = Evaluator::default();
        let refactor = refactor_assignments(
            vec![("a", true, false)],
            Vec::new(),
            &evaluator,
        );
        let expected = RefactorResult::empty()
            .with_variables(vec!["a"]);
        assert_eq!(refactor, expected);
    }
}
