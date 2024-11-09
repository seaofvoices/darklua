use blake3;
use hex;
use std::collections::HashMap;
use strfmt::strfmt;

pub struct RuntimeIdentifierBuilder {
    format: String,
    hash: String,
    keywords: Option<Vec<String>>,
}

impl RuntimeIdentifierBuilder {
    pub fn new(
        format: impl Into<String>,
        identifier: &[u8],
        keywords: Option<Vec<String>>,
    ) -> Result<Self, String> {
        let format: String = format.into();
        if !format.as_str().contains("{name}") {
            return Err("`name` field is required for runtime identifier".to_string());
        }
        let hash = blake3::hash(identifier);
        Ok(Self {
            format,
            hash: hex::encode(&hash.as_bytes()[..8]),
            keywords,
        })
    }

    pub fn build(&self, name: &str) -> Result<String, String> {
        let mut vars = HashMap::new();
        vars.insert("name".to_owned(), name);
        vars.insert("hash".to_owned(), self.hash.as_str());

        let name = strfmt(&self.format, &vars).map_err(|err| err.to_string())?;

        if let Some(keywords) = &self.keywords {
            if keywords.contains(&name) {
                Err(format!("Runtime variable `{name}` cannot be set because it contains a reserved keyword."))?;
            }
        }

        Ok(name)
    }
}
