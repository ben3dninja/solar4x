use serde::{de::Visitor, Deserialize, Deserializer};

pub fn deserialize_options<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Deserialize::deserialize(d).map(|x: Option<T>| x.unwrap_or_default())
}

pub fn deserialize_exponents<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct ExponentVisitor;

    impl<'d> Visitor<'d> for ExponentVisitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a value in scentific notation")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'d>,
        {
            let mut val = None;
            let mut exp = None;
            while let Some(key) = map.next_key::<&str>()? {
                if key.contains("Value") {
                    val = Some(map.next_value::<f64>()?);
                } else if key.contains("Exponent") {
                    exp = Some(map.next_value::<i32>()?);
                }
            }
            val.and_then(|v| exp.map(|e| Some(v * 10f64.powi(e))))
                .ok_or(serde::de::Error::custom(
                    "number could not be deserialized from map",
                ))
        }
    }
    d.deserialize_option(ExponentVisitor)
        .map(|f| f.unwrap_or_default())
}
