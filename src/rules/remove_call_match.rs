use std::ops;

use crate::nodes::{Arguments, DoStatement, Expression, Prefix, Statement, TableEntry};
use crate::process::{Evaluator, IdentifierTracker, NodeProcessor};
use crate::utils::{expressions_as_expression, expressions_as_statement};

pub(crate) trait CallMatch<T> {
    fn matches(&self, identifiers: &IdentifierTracker, prefix: &Prefix) -> bool;
}

#[derive(Default)]
pub(crate) struct RemoveFunctionCallProcessor<Args, T: CallMatch<Args>> {
    identifier_tracker: IdentifierTracker,
    evaluator: Evaluator,
    preserve_args_side_effects: bool,
    matcher: T,
    _phantom: std::marker::PhantomData<Args>,
}

impl<F> CallMatch<(&IdentifierTracker, &Prefix)> for F
where
    F: Fn(&IdentifierTracker, &Prefix) -> bool,
{
    fn matches(&self, identifiers: &IdentifierTracker, prefix: &Prefix) -> bool {
        (self)(identifiers, prefix)
    }
}

impl<F> CallMatch<&Prefix> for F
where
    F: Fn(&Prefix) -> bool,
{
    fn matches(&self, _identifiers: &IdentifierTracker, prefix: &Prefix) -> bool {
        (self)(prefix)
    }
}

impl<Args, T: CallMatch<Args>> RemoveFunctionCallProcessor<Args, T> {
    pub(crate) fn new(preserve_args_side_effects: bool, matcher: T) -> Self {
        Self {
            identifier_tracker: Default::default(),
            evaluator: Default::default(),
            preserve_args_side_effects,
            matcher,
            _phantom: Default::default(),
        }
    }

    fn preserve_side_effects(&self, arguments: &Arguments) -> Vec<Expression> {
        match arguments {
            Arguments::Tuple(tuple) => tuple
                .iter_values()
                .filter(|value| self.evaluator.has_side_effects(value))
                .cloned()
                .collect(),
            Arguments::Table(table) => {
                let mut expressions = Vec::new();

                for entry in table.iter_entries() {
                    match entry {
                        TableEntry::Field(field) => {
                            let expression = field.get_value();
                            if self.evaluator.has_side_effects(expression) {
                                expressions.push(expression.clone());
                            }
                        }
                        TableEntry::Index(index) => {
                            let key = index.get_key();
                            let value = index.get_value();

                            if self.evaluator.has_side_effects(key) {
                                expressions.push(key.clone());
                            }
                            if self.evaluator.has_side_effects(value) {
                                expressions.push(value.clone());
                            }
                        }
                        TableEntry::Value(value) => {
                            if self.evaluator.has_side_effects(value) {
                                expressions.push(value.clone());
                            }
                        }
                    }
                }

                expressions
            }
            Arguments::String(_) => Vec::new(),
        }
    }
}

impl<Args, T: CallMatch<Args>> ops::Deref for RemoveFunctionCallProcessor<Args, T> {
    type Target = IdentifierTracker;

    fn deref(&self) -> &Self::Target {
        &self.identifier_tracker
    }
}

impl<Args, T: CallMatch<Args>> ops::DerefMut for RemoveFunctionCallProcessor<Args, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.identifier_tracker
    }
}

impl<Args, T: CallMatch<Args>> NodeProcessor for RemoveFunctionCallProcessor<Args, T> {
    fn process_statement(&mut self, statement: &mut Statement) {
        if let Statement::Call(call) = statement {
            if self
                .matcher
                .matches(&self.identifier_tracker, call.get_prefix())
            {
                *statement = if self.preserve_args_side_effects {
                    expressions_as_statement(self.preserve_side_effects(call.get_arguments()))
                } else {
                    DoStatement::default().into()
                };
            }
        }
    }

    fn process_expression(&mut self, expression: &mut Expression) {
        if let Expression::Call(call) = expression {
            if self
                .matcher
                .matches(&self.identifier_tracker, call.get_prefix())
            {
                *expression = if self.preserve_args_side_effects {
                    expressions_as_expression(self.preserve_side_effects(call.get_arguments()))
                } else {
                    Expression::nil()
                };
            }
        }
    }
}
