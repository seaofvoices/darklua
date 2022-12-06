use crate::nodes::{Block, DoStatement, Expression, IfExpression, IfStatement, Statement};
use crate::process::{DefaultVisitor, Evaluator, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use std::mem;

use super::verify_no_rule_properties;

enum FilterResult {
    Keep,
    Remove,
    Replace(Block),
}

#[derive(Debug, Clone, Default)]
struct IfFilter {
    evaluator: Evaluator,
}

impl IfFilter {
    fn filter(&self, statement: &mut IfStatement) -> FilterResult {
        let found_always_true_branch = self.filter_branches(statement);

        if found_always_true_branch {
            statement.take_else_block();
        }

        let branch_count = statement.branch_count();

        if branch_count == 0 {
            if let Some(block) = statement.take_else_block() {
                if block.is_empty() {
                    FilterResult::Remove
                } else {
                    FilterResult::Replace(block)
                }
            } else {
                FilterResult::Remove
            }
        } else if found_always_true_branch && branch_count == 1 {
            let branch = statement.mutate_branches().iter_mut().next().unwrap();

            if !self.evaluator.has_side_effects(branch.get_condition()) {
                let mut branch_block = Block::default();
                mem::swap(branch.mutate_block(), &mut branch_block);

                FilterResult::Replace(branch_block)
            } else {
                FilterResult::Keep
            }
        } else {
            FilterResult::Keep
        }
    }

    fn filter_branches(&self, statement: &mut IfStatement) -> bool {
        let branches = statement.mutate_branches();
        let mut found_always_true_branch = false;
        let mut i = 0;

        while i != branches.len() {
            if found_always_true_branch {
                branches.remove(i);
            } else {
                let branch = branches.get_mut(i).unwrap();
                let condition = branch.get_condition();
                let is_truthy = self.evaluator.evaluate(condition).is_truthy();

                if let Some(is_truthy) = is_truthy {
                    if is_truthy {
                        found_always_true_branch = true;
                        i += 1;
                    } else {
                        let side_effects = self.evaluator.has_side_effects(condition);

                        if side_effects {
                            // only need to clear if there are side effects because it means
                            // that we are keeping the branch just for the condition
                            branch.mutate_block().clear();
                            i += 1;
                        } else {
                            branches.remove(i);
                        }
                    }
                } else {
                    i += 1;
                }
            }
        }

        found_always_true_branch
    }

    fn simplify_if(&self, if_expression: &mut IfExpression) -> Option<Expression> {
        let condition_value = self.evaluator.evaluate(if_expression.get_condition());
        match condition_value.is_truthy() {
            Some(true) => {
                if self
                    .evaluator
                    .has_side_effects(if_expression.get_condition())
                {
                    if_expression.clear_elseif_branches();
                    *if_expression.mutate_else_result() = Self::result_placeholder();
                    None
                } else {
                    let result = if_expression.get_result().clone();

                    Some(if self.evaluator.can_return_multiple_values(&result) {
                        result.in_parentheses()
                    } else {
                        result
                    })
                }
            }
            Some(false) => {
                if self
                    .evaluator
                    .has_side_effects(if_expression.get_condition())
                {
                    *if_expression.mutate_result() = Self::result_placeholder();
                    None
                } else if let Some(branch) = if_expression.remove_branch(0) {
                    let (new_condition, new_result) = branch.into_expressions();
                    *if_expression.mutate_condition() = new_condition;
                    *if_expression.mutate_result() = new_result;
                    self.simplify_if(if_expression)
                } else {
                    let result = if_expression.get_else_result().clone();

                    Some(if self.evaluator.can_return_multiple_values(&result) {
                        result.in_parentheses()
                    } else {
                        result
                    })
                }
            }
            None => {
                let mut keep_next_branches = true;
                let mut replace_else_with = None;

                if_expression.retain_elseif_branches_mut(|branch| {
                    keep_next_branches && {
                        let branch_condition_value =
                            self.evaluator.evaluate(branch.get_condition());
                        match branch_condition_value.is_truthy() {
                            Some(true) => {
                                keep_next_branches = false;

                                if self.evaluator.has_side_effects(branch.get_condition()) {
                                    true
                                } else {
                                    replace_else_with = Some(branch.get_result().clone());
                                    false
                                }
                            }
                            Some(false) => {
                                if self.evaluator.has_side_effects(branch.get_condition()) {
                                    *branch.mutate_result() = Self::result_placeholder();
                                    true
                                } else {
                                    false
                                }
                            }
                            None => true,
                        }
                    }
                });

                if !keep_next_branches {
                    *if_expression.mutate_else_result() =
                        replace_else_with.unwrap_or_else(Self::result_placeholder);
                }
                None
            }
        }
    }

    fn result_placeholder() -> Expression {
        Expression::nil()
    }
}

impl NodeProcessor for IfFilter {
    fn process_block(&mut self, block: &mut Block) {
        block.filter_mut_statements(|statement| {
            let result = match statement {
                Statement::If(if_statement) => self.filter(if_statement),
                _ => FilterResult::Keep,
            };

            match result {
                FilterResult::Keep => true,
                FilterResult::Remove => false,
                FilterResult::Replace(block) => {
                    *statement = DoStatement::new(block).into();
                    true
                }
            }
        });
    }

    fn process_expression(&mut self, expression: &mut Expression) {
        if let Expression::If(if_expression) = expression {
            if let Some(replace_with) = self.simplify_if(if_expression) {
                *expression = replace_with;
            }
        }
    }
}

pub const REMOVE_UNUSED_IF_BRANCH_RULE_NAME: &str = "remove_unused_if_branch";

/// A rule that removes unused if branches. It can also turn a if statement into a do block
/// statement.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveUnusedIfBranch {}

impl FlawlessRule for RemoveUnusedIfBranch {
    fn flawless_process(&self, block: &mut Block, _: &mut Context) {
        let mut processor = IfFilter::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveUnusedIfBranch {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_UNUSED_IF_BRANCH_RULE_NAME
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

    fn new_rule() -> RemoveUnusedIfBranch {
        RemoveUnusedIfBranch::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_unused_if_branch", rule);
    }
}
