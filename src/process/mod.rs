//! Defines how rules can process and mutate Lua nodes.

mod evaluator;
#[cfg(test)]
mod node_counter;
mod node_processor;
mod scope_visitor;
mod visitors;

pub use evaluator::*;
#[cfg(test)]
pub use node_counter::NodeCounter;
pub use node_processor::NodeProcessor;
pub use scope_visitor::{Scope, ScopeVisitor};
pub use visitors::{DefaultVisitor, NodeVisitor};
