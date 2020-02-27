//! Obfuscate Lua 5.1 scripts.

pub mod nodes;
pub mod process;
pub mod rules;
mod lua_generator;
mod parser;

pub use lua_generator::{LuaGenerator, ToLua};
pub use parser::Parser;

pub use luaparser::ParsingError;
