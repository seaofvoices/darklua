use serde::{Deserialize, Serialize};
use serde_with::formats::SemicolonSeparator;
use serde_with::serde_as;
use serde_with::StringWithSeparator;

use std::collections::HashMap;
use std::path::PathBuf;

/// A require mode for the default behavior of PUC Lua interpreters.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct LuaRequireMode {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    sources: HashMap<String, PathBuf>,
    #[serde_as(as = "StringWithSeparator::<SemicolonSeparator, String>")]
    #[serde(default = "get_default_lua_path")]
    lua_path: Vec<String>,
}

fn get_default_lua_path() -> Vec<String> {
    std::env::var("LUA_PATH")
        .ok()
        .map(|x| x.split(";").map(|x| x.to_owned()).collect())
        .unwrap_or(vec![
            // this is order sensitive, and adjusted to match (roughly) the path require mode
            // "./?".to_string(),
            "./?.luau".to_string(),
            "./?.lua".to_string(),
            "./?/init.luau".to_string(),
            "./?/init.lua".to_string(), // TODO: do we want to allow resource files to both overshadow eachother AND possibly code?
                                        // ex: require("ab.c") -> ./ab/c.lua, ./ab/c/init.lua, ./ab/c.{txt,json,toml,yaml}
                                        // Might actually be useful in the case you want to replace a resource file with code (just by shadowing).
                                        // These are needed (either in LUA_PATH or here) for requiring resources because you can't use . in lua
                                        // require paths.
                                        // "./?.txt".to_string(),
                                        // "./?.json".to_string(),
                                        // "./?.toml".to_string(),
                                        // "./?.yaml".to_string(),
        ])
}

impl Default for LuaRequireMode {
    fn default() -> Self {
        Self {
            lua_path: get_default_lua_path(),
            sources: Default::default(),
        }
    }
}

impl LuaRequireMode {
    pub fn new(lua_path: impl Into<String>) -> Self {
        Self {
            lua_path: lua_path.into().split(";").map(|x| x.to_owned()).collect(),
            sources: Default::default(),
        }
    }

    pub(crate) fn lua_path(&self) -> &Vec<String> {
        &self.lua_path
    }
}
