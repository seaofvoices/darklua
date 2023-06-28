use std::ops::{Deref, DerefMut};

use crate::nodes::{
    AssignStatement, BinaryExpression, Block, CompoundAssignStatement, DoStatement, Expression,
    FieldExpression, IndexExpression, LocalAssignStatement, Prefix, Statement, Variable,
};
use crate::process::{IdentifierTracker, NodeProcessor, NodeVisitor, ScopeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

struct Processor {
    identifier_tracker: IdentifierTracker,
}

impl Processor {
    #[inline]
    fn generate_variable(&mut self) -> String {
        self.identifier_tracker
            .generate_identifier_with_prefix("__DARKLUA_VAR")
    }

    fn simplify_prefix(&self, prefix: &Prefix) -> Option<Prefix> {
        match prefix {
            Prefix::Parenthese(parenthese) => {
                if let Expression::Identifier(identifier) = parenthese.inner_expression() {
                    Some(Prefix::from(identifier.clone()))
                } else {
                    None
                }
            }
            Prefix::Identifier(_) | Prefix::Call(_) | Prefix::Field(_) | Prefix::Index(_) => None,
        }
    }

    fn remove_parentheses(&self, expression: impl Into<Expression>) -> Expression {
        let expression = expression.into();
        if let Expression::Parenthese(parenthese) = expression {
            parenthese.into_inner_expression()
        } else {
            expression
        }
    }

    fn replace_with(&mut self, assignment: &CompoundAssignStatement) -> Option<Statement> {
        match assignment.get_variable() {
            Variable::Index(index) => {
                let prefix_assignment = match index.get_prefix() {
                    Prefix::Identifier(_) => None,
                    Prefix::Parenthese(parenthese)
                        if matches!(
                            parenthese.inner_expression(),
                            Expression::False(_)
                                | Expression::Identifier(_)
                                | Expression::Number(_)
                                | Expression::Nil(_)
                                | Expression::String(_)
                                | Expression::True(_)
                                | Expression::VariableArguments(_)
                        ) =>
                    {
                        None
                    }
                    Prefix::Call(_)
                    | Prefix::Field(_)
                    | Prefix::Index(_)
                    | Prefix::Parenthese(_) => Some(self.generate_variable()),
                };
                let index_assignment = match index.get_index() {
                    Expression::False(_)
                    | Expression::Identifier(_)
                    | Expression::Number(_)
                    | Expression::Nil(_)
                    | Expression::String(_)
                    | Expression::True(_)
                    | Expression::VariableArguments(_) => None,
                    Expression::Parenthese(parenthese)
                        if matches!(
                            parenthese.inner_expression(),
                            Expression::False(_)
                                | Expression::Identifier(_)
                                | Expression::Number(_)
                                | Expression::Nil(_)
                                | Expression::String(_)
                                | Expression::True(_)
                                | Expression::VariableArguments(_)
                        ) =>
                    {
                        None
                    }
                    Expression::Binary(_)
                    | Expression::Call(_)
                    | Expression::Field(_)
                    | Expression::Function(_)
                    | Expression::If(_)
                    | Expression::Index(_)
                    | Expression::Parenthese(_)
                    | Expression::Table(_)
                    | Expression::Unary(_) => Some(self.generate_variable()),
                };

                match (prefix_assignment, index_assignment) {
                    (None, None) => {
                        let variable = IndexExpression::new(
                            self.simplify_prefix(index.get_prefix())
                                .unwrap_or_else(|| index.get_prefix().clone()),
                            self.remove_parentheses(index.get_index().clone()),
                        );

                        Some(
                            AssignStatement::from_variable(
                                variable.clone(),
                                BinaryExpression::new(
                                    assignment.get_operator().to_binary_operator(),
                                    variable,
                                    assignment.get_value().clone(),
                                ),
                            )
                            .into(),
                        )
                    }
                    (None, Some(index_variable)) => {
                        let assign = LocalAssignStatement::from_variable(index_variable.clone())
                            .with_value(self.remove_parentheses(index.get_index().clone()));
                        let variable = IndexExpression::new(
                            self.simplify_prefix(index.get_prefix())
                                .unwrap_or_else(|| index.get_prefix().clone()),
                            Expression::identifier(index_variable),
                        );
                        Some(into_do_assignment(assignment, assign, variable))
                    }
                    (Some(prefix_variable), None) => {
                        let assign = LocalAssignStatement::from_variable(prefix_variable.clone())
                            .with_value(self.remove_parentheses(index.get_prefix().clone()));
                        let variable = IndexExpression::new(
                            Prefix::from_name(prefix_variable),
                            index.get_index().clone(),
                        );

                        Some(into_do_assignment(assignment, assign, variable))
                    }
                    (Some(prefix_variable), Some(index_variable)) => {
                        let assign = LocalAssignStatement::from_variable(prefix_variable.clone())
                            .with_value(self.remove_parentheses(index.get_prefix().clone()))
                            .with_variable(index_variable.clone())
                            .with_value(self.remove_parentheses(index.get_index().clone()));
                        let variable = IndexExpression::new(
                            Prefix::from_name(prefix_variable),
                            Expression::identifier(index_variable),
                        );
                        Some(into_do_assignment(assignment, assign, variable))
                    }
                }
            }
            Variable::Field(field) => match field.get_prefix() {
                Prefix::Identifier(_) => None,
                Prefix::Parenthese(parenthese)
                    if matches!(
                        parenthese.inner_expression(),
                        Expression::False(_)
                            | Expression::Identifier(_)
                            | Expression::Number(_)
                            | Expression::Nil(_)
                            | Expression::String(_)
                            | Expression::True(_)
                            | Expression::VariableArguments(_)
                    ) =>
                {
                    let new_prefix =
                        if let Expression::Identifier(identifier) = parenthese.inner_expression() {
                            Prefix::from(identifier.clone())
                        } else {
                            parenthese.clone().into()
                        };
                    let new_variable = FieldExpression::new(new_prefix, field.get_field().clone());

                    Some(
                        AssignStatement::from_variable(
                            new_variable.clone(),
                            BinaryExpression::new(
                                assignment.get_operator().to_binary_operator(),
                                new_variable,
                                assignment.get_value().clone(),
                            ),
                        )
                        .into(),
                    )
                }
                Prefix::Call(_) | Prefix::Field(_) | Prefix::Index(_) | Prefix::Parenthese(_) => {
                    let identifier = self.generate_variable();
                    let new_variable = FieldExpression::new(
                        Prefix::from_name(&identifier),
                        field.get_field().clone(),
                    );
                    Some(
                        DoStatement::new(
                            Block::default()
                                .with_statement(
                                    LocalAssignStatement::from_variable(identifier).with_value(
                                        match field.get_prefix().clone() {
                                            Prefix::Parenthese(parenthese) => {
                                                parenthese.into_inner_expression()
                                            }
                                            prefix => prefix.into(),
                                        },
                                    ),
                                )
                                .with_statement(AssignStatement::from_variable(
                                    new_variable.clone(),
                                    BinaryExpression::new(
                                        assignment.get_operator().to_binary_operator(),
                                        new_variable,
                                        assignment.get_value().clone(),
                                    ),
                                )),
                        )
                        .into(),
                    )
                }
            },
            Variable::Identifier(_) => None,
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            identifier_tracker: IdentifierTracker::new(),
        }
    }
}

impl Deref for Processor {
    type Target = IdentifierTracker;

    fn deref(&self) -> &Self::Target {
        &self.identifier_tracker
    }
}

impl DerefMut for Processor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.identifier_tracker
    }
}

impl NodeProcessor for Processor {
    fn process_statement(&mut self, statement: &mut Statement) {
        if let Statement::CompoundAssign(assignment) = statement {
            let variable = assignment.get_variable();
            *statement = self.replace_with(assignment).unwrap_or_else(|| {
                AssignStatement::from_variable(
                    variable.clone(),
                    BinaryExpression::new(
                        assignment.get_operator().to_binary_operator(),
                        variable.clone(),
                        assignment.get_value().clone(),
                    ),
                )
                .into()
            });
        }
    }
}

fn into_do_assignment(
    compound_assignment: &CompoundAssignStatement,
    assign: impl Into<Statement>,
    variable: impl Into<Variable>,
) -> Statement {
    let variable = variable.into();
    DoStatement::new(
        Block::default()
            .with_statement(assign.into())
            .with_statement(AssignStatement::from_variable(
                variable.clone(),
                BinaryExpression::new(
                    compound_assignment.get_operator().to_binary_operator(),
                    variable,
                    compound_assignment.get_value().clone(),
                ),
            )),
    )
    .into()
}

pub const REMOVE_COMPOUND_ASSIGNMENT_RULE_NAME: &str = "remove_compound_assignment";

/// A rule that converts compound assignment (like `+=`) into regular assignments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveCompoundAssignment {}

impl FlawlessRule for RemoveCompoundAssignment {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Processor::default();
        ScopeVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveCompoundAssignment {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_COMPOUND_ASSIGNMENT_RULE_NAME
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

    fn new_rule() -> RemoveCompoundAssignment {
        RemoveCompoundAssignment::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_compound_assignment", rule);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'remove_compound_assignment',
            prop: "something",
        }"#,
        );
        let err_message = match result {
            Ok(_) => panic!("expected error when deserializing rule"),
            Err(e) => e,
        }
        .to_string();
        pretty_assertions::assert_eq!(err_message, "unexpected field 'prop'");
    }
}
