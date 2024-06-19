use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use crate::utils::hash::hash;

const ID_PREFIX: &str = "https://api.le-systeme-solaire.net/rest/bodies/";

// TODO : change default id
#[derive(Default, PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Hash, Serialize)]
pub struct BodyID(u64);

impl std::fmt::Display for BodyID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl From<&str> for BodyID {
    fn from(value: &str) -> Self {
        Self(hash(&value.to_owned()))
    }
}

struct IDVisitor;

impl<'de> Visitor<'de> for IDVisitor {
    type Value = BodyID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a body id")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut id = None;
        while let Some((key, value)) = map.next_entry::<&str, &str>()? {
            if key == "rel" {
                id = strip_id_prefix(value);
            }
        }
        id.ok_or(serde::de::Error::custom(
            "id could not be deserialized from map",
        ))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        strip_id_prefix(v).ok_or(serde::de::Error::custom(
            "id could not be deserialized from string",
        ))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BodyID(v))
    }
}

fn strip_id_prefix(s: &str) -> Option<BodyID> {
    s.strip_prefix(ID_PREFIX).map(BodyID::from)
}

impl<'de> Deserialize<'de> for BodyID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IDVisitor)
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn test_single() {
        let id: BodyID = from_str(
            r#"
            "https://api.le-systeme-solaire.net/rest/bodies/terre"
        "#,
        )
        .unwrap();
        assert_eq!(id, "terre".into());
    }

    #[test]
    fn test_map() {
        let id: BodyID = from_str(
            r#"
        {
            "planet": "terre",
            "rel": "https://api.le-systeme-solaire.net/rest/bodies/terre"
        }"#,
        )
        .unwrap();
        assert_eq!(id, "terre".into());
    }
}
