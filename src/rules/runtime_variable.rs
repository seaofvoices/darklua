use blake3;
use hex;
use std::collections::HashMap;
use strfmt::strfmt;

pub const DEFAULT_RUNTIME_VARIABLE_FORMAT: &str = "{prefix}_{name}{hash}";

pub struct RuntimeVariableBuilder {
    prefix: String,
    format: String,
    hash: String,
    keywords: Option<Vec<String>>,
}

impl RuntimeVariableBuilder {
    pub fn new(
        prefix: impl Into<String>,
        format: impl Into<String>,
        identifier: &[u8],
        keywords: Option<Vec<String>>,
    ) -> Self {
        let hash = blake3::hash(identifier);
        Self {
            prefix: prefix.into(),
            format: format.into(),
            hash: hex::encode(&hash.as_bytes()[..8]),
            keywords,
        }
    }

    pub fn build(&self, name: &str) -> Result<String, String> {
        let mut vars = HashMap::new();
        vars.insert("prefix".to_string(), self.prefix.as_str());
        vars.insert("name".to_string(), name);
        vars.insert("hash".to_string(), self.hash.as_str());

        let name = strfmt(&self.format, &vars).map_err(|err| err.to_string())?;

        if let Some(keywords) = &self.keywords {
            if keywords.contains(&name) {
                Err(format!("Runtime variable `{name}` cannot be set because it contains a reserved keyword."))?;
            }
        }

        Ok(name)
    }
}
