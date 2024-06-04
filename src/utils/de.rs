use serde::{Deserialize, Deserializer};

pub fn deserialize_options<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Deserialize::deserialize(d).map(|x: Option<T>| x.unwrap_or_default())
}
