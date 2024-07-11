use arrayvec::ArrayString;
use serde::{de::Visitor, Deserialize, Deserializer};

use crate::MAX_ID_LENGTH;

const ID_PREFIX: &str = "https://api.le-systeme-solaire.net/rest/bodies/";

#[derive(PartialEq, Debug, Clone)]
pub struct MainBodyID(pub String);

impl From<&str> for MainBodyID {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
pub type BodyID = ArrayString<MAX_ID_LENGTH>;

pub fn id_from(s: &str) -> BodyID {
    BodyID::from(s).unwrap()
}

impl From<MainBodyID> for BodyID {
    fn from(value: MainBodyID) -> Self {
        id_from(&value.0)
    }
}

struct IDVisitor;

impl<'de> Visitor<'de> for IDVisitor {
    type Value = MainBodyID;

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
}

fn strip_id_prefix(s: &str) -> Option<MainBodyID> {
    s.strip_prefix(ID_PREFIX)
        .map(MainBodyID::from)
        .filter(|s| s.0.len() < MAX_ID_LENGTH)
}

impl<'de> Deserialize<'de> for MainBodyID {
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
        let id: MainBodyID = from_str(
            r#"
            "https://api.le-systeme-solaire.net/rest/bodies/terre"
        "#,
        )
        .unwrap();
        assert_eq!(id, "terre".into());
    }

    #[test]
    fn test_map() {
        let id: MainBodyID = from_str(
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
