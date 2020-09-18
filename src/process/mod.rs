//! Defines how rules can process and mutate Lua nodes.

mod evaluator;
#[cfg(test)]
mod node_counter;
mod node_processor;
pub mod processors;
mod scope_visitor;
mod visitors;

pub use evaluator::*;
#[cfg(test)]
pub use node_counter::NodeCounter;
pub use node_processor::{NodeProcessor, NodeProcessorMut};
pub use scope_visitor::{Scope, ScopeMut, ScopeVisitor, ScopeVisitorMut};
pub use visitors::{DefaultVisitor, DefaultVisitorMut, NodeVisitor, NodeVisitorMut};
