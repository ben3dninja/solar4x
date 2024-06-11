use serde::{de::Visitor, Deserialize, Deserializer};

const ID_PREFIX: &str = "https://api.le-systeme-solaire.net/rest/bodies/";

// TODO : change default id
#[derive(Default, PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Hash)]
pub struct BodyID(String);

impl std::fmt::Display for BodyID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl From<&str> for BodyID {
    fn from(value: &str) -> Self {
        Self(value.into())
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
        let mut id = BodyID::default();
        while let Some((key, value)) = map.next_entry::<&str, &str>()? {
            if key == "rel" {
                id = strip_id_prefix(value);
            }
        }
        Ok(id)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(strip_id_prefix(v))
    }
}

fn strip_id_prefix(s: &str) -> BodyID {
    // TODO : change default
    if let Some(s) = s.strip_prefix(ID_PREFIX) {
        BodyID(s.to_owned())
    } else {
        BodyID::default()
    }
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
        assert_eq!(id.0, "terre");
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
        assert_eq!(id.0, "terre");
    }
}
