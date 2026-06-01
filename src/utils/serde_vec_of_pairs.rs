use std::{cmp, fmt, marker::PhantomData};

use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};

pub(crate) fn deserialize_vec_of_pairs<'de, D, K, V>(
    deserializer: D,
) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    struct MapAsVecVisitor<K, V>(PhantomData<Vec<(K, V)>>);

    impl<K, V> MapAsVecVisitor<K, V> {
        fn new() -> Self {
            MapAsVecVisitor(PhantomData)
        }
    }

    impl<'de, K, V> Visitor<'de> for MapAsVecVisitor<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        type Value = Vec<(K, V)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Vec<(K, V)>, E> {
            Ok(Vec::new())
        }

        #[inline]
        fn visit_map<T>(self, mut access: T) -> Result<Vec<(K, V)>, T::Error>
        where
            T: MapAccess<'de>,
        {
            let mut values = Vec::with_capacity(cmp::min(access.size_hint().unwrap_or(0), 4096));

            while let Some((key, value)) = access.next_entry()? {
                values.push((key, value));
            }

            Ok(values)
        }
    }

    let visitor = MapAsVecVisitor::<K, V>::new();

    deserializer.deserialize_map(visitor)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestVecOfPairs {
        #[serde(deserialize_with = "deserialize_vec_of_pairs")]
        items: Vec<(String, usize)>,
    }

    #[test]
    fn deserialize_empty_map() {
        let v: TestVecOfPairs = serde_json::from_str(r#"{"items": {}}"#).unwrap();
        assert_eq!(v.items, Vec::new());
    }

    #[test]
    fn deserialize_map_with_two_pairs() {
        let v: TestVecOfPairs =
            serde_json::from_str(r#"{"items": {"key": 1, "key2": 2}}"#).unwrap();

        assert_eq!(v.items, vec![("key".to_owned(), 1), ("key2".to_owned(), 2)]);
    }
}
