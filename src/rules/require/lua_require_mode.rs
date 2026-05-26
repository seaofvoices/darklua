use serde::{Deserialize, Serialize};
use serde_with::formats::SemicolonSeparator;
use serde_with::serde_as;
use serde_with::StringWithSeparator;

const DEFAULT_PATH_VARIABLE: &str = "LUA_PATH";
const DEFAULT_SEARCH_PATHS: [&str; 4] = [
    // this is order sensitive, and adjusted to match the path require mode
    "./?.luau",
    "./?.lua",
    "./?/init.luau",
    "./?/init.lua",
];

/// A require mode for the default behavior of PUC Lua interpreters.
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct LuaRequireMode {
    env: Option<String>,
    #[serde_as(as = "Option<StringWithSeparator::<SemicolonSeparator, String>>")]
    path: Option<Vec<String>>,
}

impl LuaRequireMode {
    pub fn new(path: Option<impl Into<String>>, env: Option<impl Into<String>>) -> Self {
        Self {
            env: env.map(|x| x.into()),
            path: path.map(|x| x.into().split(";").map(|x| x.to_owned()).collect()),
        }
    }

    pub(crate) fn path(&self) -> Vec<String> {
        if let Some(path) = self.path.as_ref() {
            path.clone()
        } else {
            std::env::var(
                self.env
                    .clone()
                    .unwrap_or(DEFAULT_PATH_VARIABLE.to_string()),
            )
            .ok()
            .map(|x| x.split(";").map(|x| x.to_string()).collect())
            .unwrap_or(DEFAULT_SEARCH_PATHS.iter().map(|x| x.to_string()).collect())
        }
    }
}
