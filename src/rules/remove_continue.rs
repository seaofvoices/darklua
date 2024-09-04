use std::fmt::Debug;

use crate::nodes::{
    AssignStatement, Block, Expression, Identifier, IfStatement, LastStatement,
    LocalAssignStatement, RepeatStatement, Statement, TypedIdentifier, Variable,
};
use crate::process::{DefaultVisitor, NodeProcessor, NodeVisitor};
use crate::rules::{
    Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties,
};

use super::verify_no_rule_properties;

use blake3;
use hex;

const BREAK_VARIABLE_NAME: &str = "__DARKLUA_REMOVE_CONTINUE_break";

#[derive(Default)]
struct Processor {
    break_variable_name: String,
}

fn continue_break_exists(block: &Block) -> (bool, bool) {
    let (mut has_continue, mut has_break) = if let Some(last_stmt) = block.get_last_statement() {
        (
            matches!(last_stmt, LastStatement::Continue(_)),
            matches!(last_stmt, LastStatement::Break(_)),
        )
    } else {
        (false, false)
    };
    for stmt in block.iter_statements() {
        match stmt {
            Statement::If(if_stmt) => {
                for branch in if_stmt.iter_branches() {
                    let (c, b) = continue_break_exists(branch.get_block());
                    has_continue = has_continue || c;
                    has_break = has_break || b;
                }
            }
            Statement::Do(do_stmt) => {
                let (c, b) = continue_break_exists(do_stmt.get_block());
                has_continue = has_continue || c;
                has_break = has_break || b;
            }
            _ => {}
        }
        if has_continue && has_break {
            break;
        }
    }
    (has_continue, has_break)
}

fn continues_to_breaks(block: &mut Block) {
    if let Some(last_stmt) = block.mutate_last_statement() {
        if matches!(last_stmt, LastStatement::Continue(_)) {
            *last_stmt = LastStatement::new_break();
        }
    }
    for stmt in block.iter_mut_statements() {
        match stmt {
            Statement::If(if_stmt) => {
                for branch in if_stmt.mutate_branches().iter_mut() {
                    continues_to_breaks(branch.mutate_block());
                }
            }
            Statement::Do(do_stmt) => {
                continues_to_breaks(do_stmt.mutate_block());
            }
            _ => {}
        }
    }
}

fn continues_with_breaks_to_breaks(block: &mut Block, break_variable_name: &str) {
    if let Some(last_stmt) = block.mutate_last_statement() {
        match last_stmt {
            LastStatement::Continue(_) => {
                block.set_last_statement(LastStatement::new_break());
            }
            LastStatement::Break(_) => {
                let var = Variable::new(break_variable_name);
                let break_var_assign_stmt = AssignStatement::from_variable(var, true);
                block.push_statement(break_var_assign_stmt);
                block.set_last_statement(LastStatement::new_break());
            }
            _ => {}
        }
    }
    for stmt in block.iter_mut_statements() {
        match stmt {
            Statement::If(if_stmt) => {
                for branch in if_stmt.mutate_branches().iter_mut() {
                    continues_with_breaks_to_breaks(branch.mutate_block(), break_variable_name);
                }
            }
            Statement::Do(do_stmt) => {
                continues_with_breaks_to_breaks(do_stmt.mutate_block(), break_variable_name);
            }
            _ => {}
        }
    }
}

impl Processor {
    fn process(&self, block: &mut Block) {
        let (has_continue, has_break) = continue_break_exists(block);

        if has_continue {
            let (mut stmts, break_variable_handler) = if has_break {
                let var = TypedIdentifier::new(self.break_variable_name.as_str());
                let value = Expression::False(None);
                let local_assign_stmt = LocalAssignStatement::new(vec![var], vec![value]);

                let break_block = Block::new(vec![], Some(LastStatement::new_break()));
                let break_variable_handler = IfStatement::create(
                    Identifier::new(self.break_variable_name.as_str()),
                    break_block,
                );
                continues_with_breaks_to_breaks(block, self.break_variable_name.as_str());
                (vec![local_assign_stmt.into()], Some(break_variable_handler))
            } else {
                continues_to_breaks(block);
                (Vec::new(), None)
            };
            let repeat_stmt = RepeatStatement::new(block.clone(), true);
            stmts.push(repeat_stmt.into());
            if let Some(break_variable_handler) = break_variable_handler {
                stmts.push(break_variable_handler.into());
            }
            *block = Block::new(stmts, None);
        }
    }
}

impl NodeProcessor for Processor {
    fn process_statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::NumericFor(numeric_for) => self.process(numeric_for.mutate_block()),
            Statement::GenericFor(generic_for) => self.process(generic_for.mutate_block()),
            Statement::Repeat(repeat_stmt) => self.process(repeat_stmt.mutate_block()),
            Statement::While(while_stmt) => self.process(while_stmt.mutate_block()),
            _ => (),
        }
    }
}

pub const REMOVE_CONTINUE_RULE_NAME: &str = "remove_continue";

/// A rule that removes continue statements and convert into breaks.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveContinue {}

impl FlawlessRule for RemoveContinue {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let hash = blake3::hash(format!("{block:?}").as_bytes());
        let hash_hex = hex::encode(&hash.as_bytes()[..8]);
        let break_variable_name = BREAK_VARIABLE_NAME.to_string() + hash_hex.as_str();
        let mut processor = Processor {
            break_variable_name,
        };
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
