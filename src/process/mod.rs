//! Defines how rules can process and mutate Lua nodes.

#[cfg(test)]
mod node_counter;
mod node_processor;
mod visitors;

#[cfg(test)]
pub use node_counter::NodeCounter;
pub use node_processor::NodeProcessor;
pub use visitors::{DefaultVisitor, NodeVisitor};
