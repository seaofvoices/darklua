use crate::nodes::{
    self, Block, Expression, LastStatement, LocalAssignStatement, RepeatStatement, Statement,
    TypedIdentifier,
};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

const BREAK_VARIABLE_NAME: &str = "__DARKLUA_REMOVE_CONTINUE_break";

/// deeply searches for blocks and checks if LastStatement exists
fn last_stmt_exists(block: &mut Block) -> Option<&mut Block> {
    if let Some(_) = block.get_last_statement() {
        return Some(block);
    }
    for stmt in block.iter_mut_statements() {
        match stmt {
            Statement::If(if_stmt) => {
                for v in if_stmt.mutate_branches() {
                    if let Some(b) = last_stmt_exists(v.mutate_block()) {
                        return Some(b);
                    }
                }
                return None;
            }
            Statement::Repeat(repeat) => {
                if let Some(b) = last_stmt_exists(repeat.mutate_block()) {
                    return Some(b);
                }
            }
            _ => {
                return None;
            }
        }
    }
    None
}

#[derive(Default)]
struct Processor {}

// impl NodeProcessor for Processor {
//     fn process_statement(&mut self, statement: &mut nodes::Statement) {
//         match statement {
//             Statement::NumericFor(numeric_for) => {
//                 let numeric_for2 = numeric_for.clone();
//                 let original_block = numeric_for.mutate_block();
//                 let new_block = if let Some(block) = last_stmt_exists(original_block) {
//                     if !matches!(block.get_last_statement(), Some(LastStatement::Continue(_))) {
//                         return
//                     }
//                     let repeat = RepeatStatement::new(numeric_for2.get_block().clone(), true);
//                     let repeat_last_stmt = if let Some(s) = numeric_for2.get_block().get_last_statement() {
//                         Some(s.clone())
//                     } else {
//                         None
//                     };
//                     Some(Block::new(vec![repeat.into()], repeat_last_stmt))
//                 } else {
//                     None
//                 };
//                 if let Some(b) = new_block {
//                     *original_block = b;
//                     if let Some(block) = last_stmt_exists(original_block) {
//                         block.set_last_statement(LastStatement::new_break());
//                     }
//                 }
//             },
//             _ => {}
//         }
//     }
// }

struct CollectedStatement {
    block: Block,
    last_statement: LastStatement,
}

fn collect_last_statements(block: &mut Block, result: &mut Vec<CollectedStatement>) {
    if let Some(last_stmt) = block.get_last_statement() {
        result.push(CollectedStatement {
            block: block.clone(),
            last_statement: last_stmt.clone(),
        });
    }
    for stmt in block.iter_mut_statements() {
        match stmt {
            Statement::If(if_stmt) => {
                for b in if_stmt.mutate_all_blocks() {
                    collect_last_statements(b, result);
                }
            }
            _ => {}
        }
    }
}

fn to_breaks() {}

fn to_breaks_with_break() {}

impl NodeProcessor for Processor {
    fn process_statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::NumericFor(numeric_for) => {
                let block = numeric_for.mutate_block();
                let last_stmts = &mut Vec::new();
                collect_last_statements(block, last_stmts);

                let has_continue = last_stmts
                    .iter()
                    .any(|cs| matches!(cs.last_statement, LastStatement::Continue(_)));
                let has_break = last_stmts
                    .iter()
                    .any(|cs| matches!(cs.last_statement, LastStatement::Break(_)));

                if has_continue {
                    let mut stmts = Vec::new();
                    if has_break {
                        let var = TypedIdentifier::new(BREAK_VARIABLE_NAME);
                        let value = Expression::False(None);
                        let local_assign_stmt = LocalAssignStatement::new(vec![var], vec![value]);
                        stmts.push(local_assign_stmt.into());
                    }
                    let repeat_stmt = RepeatStatement::new(block.clone(), true);
                    stmts.push(repeat_stmt.into());
                    let last_stmt = if let Some(l) = block.mutate_last_statement() {
                        Some(l.clone())
                    } else {
                        None
                    };
                    *block = Block::new(stmts, last_stmt);
                }
            }
            _ => {}
        }
    }
}

// fn process(block: &mut Block) {

// }

// impl NodeProcessor for Processor {
// 	fn process_statement(&mut self, statement: &mut Statement) {
// 		match statement {
// 			Statement::NumericFor(numeric_for) => {

// 			},
// 			_ => {}
// 		}
// 	}
// }

pub const REMOVE_CONTINUE_RULE_NAME: &str = "remove_continue";

/// A rule that removes trailing `nil` in local assignments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveContinue {}

impl FlawlessRule for RemoveContinue {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Processor::default();
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveContinue {
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        verify_no_rule_properties(&properties)?;

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_CONTINUE_RULE_NAME
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

    fn new_rule() -> RemoveContinue {
        RemoveContinue::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_continue", rule);
    }

    #[test]
    fn configure_with_extra_field_error() {
        let result = json5::from_str::<Box<dyn Rule>>(
            r#"{
            rule: 'remove_continue',
            prop: "something",
        }"#,
        );
        pretty_assertions::assert_eq!(result.unwrap_err().to_string(), "unexpected field 'prop'");
    }
}
