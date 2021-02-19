use crate::nodes::{
    Expression,
    AssignStatement,
    Statement,
    LastStatement,
    LocalFunctionStatement,
    Prefix,
    Variable,
};
use crate::process::{
    NodeProcessor,
    Scope,
};

use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct FindVariable {
    processing_assignment: bool,
    usage_found: bool,
    identifier: String,
    identifiers: Vec<HashSet<String>>,
}

impl FindVariable {
    pub fn new<S: Into<String>>(identifier: S) -> Self {
        Self {
            processing_assignment: false,
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

    fn is_reading_in_variable(&self, variable: &Variable) -> bool {
        match variable {
            Variable::Identifier(_) => false,
            Variable::Field(field) => {
                self.is_reading_in_prefix(field.get_prefix(), true)
            }
            Variable::Index(index) => {
                self.is_reading_in_prefix(index.get_prefix(), true)
                || self.is_reading_in_expression(index.get_index(), false)
            }
        }
    }

    fn is_reading_in_expression(&self, expression: &Expression, is_assignment: bool) -> bool {
        match expression {
            Expression::Identifier(identifier) => {
                if is_assignment {
                    false
                } else {
                    identifier == &self.identifier
                }
            }
            Expression::Field(field) => {
                self.is_reading_in_prefix(field.get_prefix(), is_assignment)
            }
            Expression::Index(index) => {
                self.is_reading_in_prefix(index.get_prefix(), is_assignment)
                || self.is_reading_in_expression(index.get_index(), false)
            }
            Expression::Parenthese(expression) => {
                self.is_reading_in_expression(expression, is_assignment)
            }
            _ => false,
        }
    }

    fn is_reading_in_prefix(&self, prefix: &Prefix, is_assignment: bool) -> bool {
        match prefix {
            Prefix::Identifier(identifier) => {
                if is_assignment {
                    false
                } else {
                    identifier == &self.identifier
                }
            }
            Prefix::Field(field) => {
                self.is_reading_in_prefix(field.get_prefix(), is_assignment)
            }
            Prefix::Index(index) => {
                self.is_reading_in_prefix(index.get_prefix(), is_assignment)
                || self.is_reading_in_expression(index.get_index(), false)
            }
            Prefix::Parenthese(expression) => {
                self.is_reading_in_expression(expression, true)
            }
            Prefix::Call(call) => {
                self.is_reading_in_prefix(call.get_prefix(), false)
            }
        }
    }
}

impl NodeProcessor for FindVariable {
    fn process_variable_expression(&mut self, variable: &String) {
        if !self.usage_found && !self.processing_assignment {
            if variable == &self.identifier && !self.is_identifier_used(variable) {
                self.usage_found = true;
            }
        }
    }

    fn process_assign_statement(&mut self, assign: &AssignStatement) {
        if !self.usage_found {
            self.processing_assignment = true;
            self.usage_found = assign.get_variables().iter()
                .any(|variable| self.is_reading_in_variable(variable));
        }
    }

    fn process_statement(&mut self, _: &Statement) {
        self.processing_assignment = false;
    }

    fn process_last_statement(&mut self, _: &LastStatement) {
        self.processing_assignment = false;
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
