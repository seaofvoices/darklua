use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(value: OneOrMany<T>) -> Self {
        match value {
            OneOrMany::One(element) => vec![element],
            OneOrMany::Many(array) => array,
        }
    }
}

pub(crate) fn deserialize_one_or_many<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let helper = OneOrMany::<T>::deserialize(deserializer)?;
    Ok(helper.into())
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestOneOrMany {
        #[serde(deserialize_with = "deserialize_one_or_many")]
        items: Vec<String>,
    }

    #[test]
    fn deserialize_single_value() {
        let v: TestOneOrMany = serde_json::from_str(r#"{"items": "only"}"#).unwrap();
        assert_eq!(v.items, vec!["only"]);
    }

    #[test]
    fn deserialize_many_values() {
        let v: TestOneOrMany = serde_json::from_str(r#"{"items": ["a", "b", "c"]}"#).unwrap();
        assert_eq!(v.items, vec!["a", "b", "c"]);
    }
}
