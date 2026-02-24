use std::path::Path;

use serde::{de, Deserialize, Deserializer, Serialize};
use wax::{Glob, Pattern};

use crate::DarkluaError;

#[derive(Debug, Clone, Serialize)]
#[serde(into = "String")]
pub(crate) struct FilterPattern {
    original: String,
    glob: Glob<'static>,
}

impl PartialEq for FilterPattern {
    fn eq(&self, other: &Self) -> bool {
        self.original == other.original
    }
}

impl Eq for FilterPattern {}

impl FilterPattern {
    pub(crate) fn new(pattern: String) -> Result<Self, DarkluaError> {
        let glob = Glob::new(&pattern)
            .map(Glob::into_owned)
            .map_err(|err| DarkluaError::invalid_glob_pattern(&pattern, err.to_string()))?;
        Ok(Self {
            original: pattern,
            glob,
        })
    }

    pub(crate) fn matches(&self, path: &Path) -> bool {
        self.glob.is_match(path)
    }

    pub(crate) fn original(&self) -> &str {
        &self.original
    }
}

impl<'de> Deserialize<'de> for FilterPattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

impl From<FilterPattern> for String {
    fn from(filter: FilterPattern) -> String {
        filter.original
    }
}
