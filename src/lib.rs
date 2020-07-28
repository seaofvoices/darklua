//! Obfuscate Lua 5.1 scripts.

pub mod generator;
pub mod nodes;
pub mod process;
pub mod rules;
mod parser;

pub use parser::Parser;

pub use luaparser::ParsingError;
