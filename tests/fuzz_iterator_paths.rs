use std::time::Duration;

use darklua_core::{
    generator::{LuaGenerator, ReadableLuaGenerator},
    nodes::{AnyNodeRef, Block, LastStatement, Statement},
    process::{
        path::{NodePath, NodePathSlice},
        visit_statements,
    },
};

mod fuzz;
mod utils;

use fuzz::*;

fn get_fuzz_duration() -> Duration {
    let millis = option_env!("PATH_FUZZ_DURATION_MILLISECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1500);

    Duration::from_millis(millis)
}

fn verify_visit_statements(mut context: FuzzContext) {
    let block = Block::fuzz(&mut context);

    visit_statements(&block, |statement: &Statement, path: &NodePathSlice| {
        if let Some(node_ref) = path.resolve(&block) {
            pretty_assertions::assert_eq!(node_ref, AnyNodeRef::from(statement));
        } else {
            report_unresolved_path(path, &block);
        }
    });
}

fn verify_visit_last_statements(mut context: FuzzContext) {
    let block = Block::fuzz(&mut context);

    visit_statements(&block, |statement: &LastStatement, path: &NodePathSlice| {
        if let Some(node_ref) = path.resolve(&block) {
            pretty_assertions::assert_eq!(node_ref, AnyNodeRef::from(statement));
        } else {
            report_unresolved_path(path, &block);
        }
    });
}

fn report_unresolved_path(path: &NodePathSlice, block: &Block) {
    let mut parent = path;
    let mut last_resolved_path = None;

    while let Some(current_parent) = parent.parent() {
        if current_parent.resolve(&block).is_some() {
            last_resolved_path = Some(current_parent);
            break;
        }
        parent = current_parent;
    }

    let mut generator = ReadableLuaGenerator::new(80);
    generator.write_block(&block);

    panic!(
        "unable to resolve path `{}` in block{}:\n```\n{}\n```{}",
        path.to_string(),
        if let Some(resolved) = last_resolved_path {
            format!(" (last resolved: `{}`)", resolved.to_string())
        } else {
            "".to_owned()
        },
        generator.into_string(),
        if let Some(resolved) = last_resolved_path {
            format!("\nLast resolved: {:#?})", resolved.resolve(&block).unwrap())
        } else {
            "".to_owned()
        },
    )
}

#[test]
fn fuzz_visit_statements_tiny_block() {
    utils::run_for_minimum_time(
        || verify_visit_statements(FuzzContext::new(2, 8)),
        get_fuzz_duration(),
    );
}

#[test]
fn fuzz_visit_last_statements_tiny_block() {
    utils::run_for_minimum_time(
        || verify_visit_last_statements(FuzzContext::new(2, 8)),
        get_fuzz_duration(),
    );
}

#[test]
fn fuzz_visit_statements_small_block() {
    utils::run_for_minimum_time(
        || verify_visit_statements(FuzzContext::new(20, 40)),
        get_fuzz_duration(),
    );
}

#[test]
fn fuzz_visit_last_statements_small_block() {
    utils::run_for_minimum_time(
        || verify_visit_last_statements(FuzzContext::new(20, 40)),
        get_fuzz_duration(),
    );
}
