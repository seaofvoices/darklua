//! Obfuscate Lua 5.1 scripts.

pub mod nodes;
mod lua_generator;
mod parser;

pub use lua_generator::{LuaGenerator, ToLua};
pub use parser::Parser;

pub use luaparser::ParsingError;
