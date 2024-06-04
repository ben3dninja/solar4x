use serde::{de::Visitor, Deserialize, Deserializer};

const ID_PREFIX: &'static str = "https://api.le-systeme-solaire.net/rest/bodies/";

// TODO : change default id
#[derive(Default, PartialEq, Debug)]
pub struct BodyID(pub String);

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
        if let Some((key, value)) = map.next_entry::<&'de str, &'de str>()? {
            if key == "rel" {
                return Ok(strip_id_prefix(value));
            }
        }
        return Ok(BodyID::default());
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        return Ok(strip_id_prefix(&v));
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

mod tests {

    use serde_json::from_str;

    use super::*;

    #[test]
    fn test_single() {
        let id: BodyID = from_str(
            r#"
        {
            "rel": "https://api.le-systeme-solaire.net/rest/bodies/terre"
        }"#,
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
            "rel": "https://api.le-systeme-solaire.net/rest/bodies/terre",
        }"#,
        )
        .unwrap();
        assert_eq!(id.0, "terre");
    }
}
