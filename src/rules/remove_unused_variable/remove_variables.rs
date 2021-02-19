use crate::nodes::{
    Block,
    AssignStatement,
    DoStatement,
    Expression,
    LocalFunctionStatement,
    Statement,
    Variable,
};
use crate::process::{
    Evaluator,
    NodeProcessorMut,
    Scope,
};
use super::refactor_assignments::refactor_assignments;

use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct RemoveVariables {
    variables: HashSet<String>,
    identifiers: Vec<HashSet<String>>,
    evaluator: Evaluator,
}

impl RemoveVariables {
    pub fn new(variables: Vec<String>) -> Self {
        Self {
            variables: HashSet::from_iter(variables.into_iter()),
            identifiers: Vec::new(),
            evaluator: Evaluator::default(),
        }
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

    #[inline]
    fn is_identifier_used(&self, identifier: &String) -> bool {
        self.identifiers.iter()
            .any(|set| set.contains(identifier))
    }

    #[inline]
    fn should_remove_identifier(&self, identifier: &String) -> bool {
        println!("  should remove {:?}", identifier);
        self.variables.contains(identifier) &&
            !self.is_identifier_used(identifier)
    }
}

impl Scope for RemoveVariables {
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

impl NodeProcessorMut for RemoveVariables {
    fn process_statement(&mut self, statement: &mut Statement) {
        let replace_with = match statement {
            Statement::Assign(assign) => {
                let values = assign.get_values();
                let variables = assign.get_variables();
                let assignments: Vec<(Variable, bool)> = variables.iter()
                    .map(|variable| {
                        println!("TEST VARIABLE = {:?}", variable);
                        let should_remove = variable.get_root_identifier()
                            .map(|identifier| self.should_remove_identifier(identifier))
                            .unwrap_or(false);

                        (variable.clone(), !should_remove)
                    })
                    .collect();

                let mut refactor = refactor_assignments(
                    assignments,
                    values.into_iter().collect(),
                    &self.evaluator,
                );

                if refactor.values.is_empty() {
                    if refactor.statements.is_empty() {
                        Some(DoStatement::default().into())
                    } else {
                        if refactor.statements.len() == 1 {
                            Some(refactor.statements.pop().unwrap())
                        } else {
                            Some(
                                DoStatement::new(
                                    Block::new(refactor.statements, None)
                                ).into()
                            )
                        }
                    }
                } else {
                    Some(
                        AssignStatement::new(refactor.variables, refactor.values).into()
                    )
                }
            }
            Statement::Function(function) => {
                let root_name = function.get_name().get_identifier();

                if self.should_remove_identifier(root_name) {
                    Some(DoStatement::default().into())
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(replace_statement) = replace_with {
            *statement = replace_statement;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nodes::{
        Block,
        AssignStatement,
        FunctionStatement,
        Variable,
    };
    use crate::process::{
        DefaultVisitorMut,
        NodeVisitorMut,
    };

    #[test]
    fn remove_assign_statement() {
        let mut processor = RemoveVariables::new(vec!["foo".to_owned()]);

        let mut block = Block::default()
            .with_statement(AssignStatement::new(
                vec![Variable::new("foo")],
                vec![Expression::Nil],
            ));

        DefaultVisitorMut::visit_block(&mut block, &mut processor);

        let expected_block = Block::default()
            .with_statement(DoStatement::default());
        assert_eq!(block, expected_block);
    }

    #[test]
    fn remove_function_statement() {
        let mut processor = RemoveVariables::new(vec!["foo".to_owned()]);

        let mut block = Block::default()
            .with_statement(FunctionStatement::from_name(
                "foo",
                Block::default(),
            ));

        DefaultVisitorMut::visit_block(&mut block, &mut processor);

        let expected_block = Block::default()
            .with_statement(DoStatement::default());
        assert_eq!(block, expected_block);
    }
}
