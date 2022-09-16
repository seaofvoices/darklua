//! Defines how rules can process and mutate Lua nodes.

mod evaluator;
pub mod mutation;
mod new_visitor;
#[cfg(test)]
mod node_counter;
mod node_processor;
pub mod path;
pub mod processors;
mod scope_visitor;
pub(crate) mod utils;
mod visitors;

pub use evaluator::*;
pub use new_visitor::{visit_statements, visit_statements_with_context, StatementVisitor};
#[cfg(test)]
pub use node_counter::NodeCounter;
pub use node_processor::NodeProcessor;
pub use scope_visitor::{Scope, ScopeVisitor};
pub use visitors::{DefaultVisitor, NodeVisitor};
