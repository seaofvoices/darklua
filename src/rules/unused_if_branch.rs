use crate::nodes::{Block, DoStatement, IfStatement, Statement};
use crate::process::{DefaultVisitorMut, Evaluator, NodeProcessorMut, NodeVisitorMut};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties};

use std::mem;

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
            if {
                if found_always_true_branch {
                    false
                } else {
                    let branch = branches.get_mut(i).unwrap();
                    let condition = branch.get_condition();
                    let is_truthy = self.evaluator.evaluate(&condition).is_truthy();

                    if let Some(is_truthy) = is_truthy {
                        if is_truthy {
                            found_always_true_branch = true;
                            true
                        } else {
                            let side_effects = self.evaluator.has_side_effects(&condition);

                            if side_effects {
                                // only need to clear if there are side effects because it means
                                // that we are keeping the branch just for the condition
                                branch.mutate_block().clear();
                            }

                            side_effects
                        }
                    } else {
                        true
                    }
                }
            } {
                i += 1;
            } else {
                branches.remove(i);
            }
        }

        found_always_true_branch
    }
}

impl NodeProcessorMut for IfFilter {
    fn process_block(&mut self, block: &mut Block) {
        block.filter_statements(|statement| {
            let result = match statement {
                Statement::If(if_statement) => self.filter(if_statement),
                _ => FilterResult::Keep,
            };

            match result {
                FilterResult::Keep => true,
                FilterResult::Remove => false,
                FilterResult::Replace(block) => {
                    mem::replace(statement, DoStatement::new(block).into());
                    true
                }
            }
        });
    }
}

pub const REMOVE_UNUSED_IF_BRANCH_RULE_NAME: &'static str = "remove_unused_if_branch";

/// A rule that removes unused if branches. It can also turn a if statement into a do block
/// statement.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveUnusedIfBranch {}

impl Rule for RemoveUnusedIfBranch {
    fn process(&self, block: &mut Block) {
        let mut processor = IfFilter::default();
        DefaultVisitorMut::visit_block(block, &mut processor);
    }

    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

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
