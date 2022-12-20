//! Defines how rules can process and mutate Lua nodes.

mod evaluator;
#[cfg(test)]
mod node_counter;
mod node_processor;
pub mod processors;
mod scope_visitor;
pub(crate) mod utils;
mod visitors;

pub use evaluator::*;
#[cfg(test)]
pub use node_counter::NodeCounter;
pub use node_processor::NodeProcessor;
pub(crate) use scope_visitor::IdentifierTracker;
pub use scope_visitor::{Scope, ScopeVisitor};
pub use visitors::{DefaultVisitor, NodeVisitor};
