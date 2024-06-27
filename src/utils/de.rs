use std::{fs::File, io::Read};

use serde::{de::Visitor, Deserialize, Deserializer};

use crate::bodies::body_data::{BodyData, MainBodyData};

const MAIN_OBJECT_FILE_PATH: &str = "main_objects.json";
const SUN_ID: &str = "soleil";

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

pub fn read_main_bodies() -> std::io::Result<Vec<BodyData>> {
    let mut file = File::open(MAIN_OBJECT_FILE_PATH)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    #[derive(Deserialize)]
    struct Input {
        bodies: Vec<MainBodyData>,
    }
    let input: Input = serde_json::from_str(&buf).map_err(std::io::Error::from)?;
    fix_bodies(input.bodies).map(|b| b.into_iter().map(BodyData::from).collect())
}

fn fix_bodies(mut bodies: Vec<MainBodyData>) -> std::io::Result<Vec<MainBodyData>> {
    bodies
        .iter_mut()
        .find(|data| data.id == SUN_ID.into())
        .ok_or(std::io::Error::other("no sun"))?
        .orbiting_bodies = bodies
        .iter()
        .filter(|data| data.host_body.is_none() && data.id != SUN_ID.into())
        .map(|planet| planet.id)
        .collect();
    bodies
        .iter_mut()
        .filter(|data| data.host_body.is_none() && data.id != SUN_ID.into())
        .for_each(|body| body.host_body = Some(SUN_ID.into()));
    Ok(bodies)
}

#[cfg(test)]
mod tests {
    use crate::{bodies::body_data::BodyType, utils::de::SUN_ID};

    use super::read_main_bodies;

    #[test]
    fn test_read_main_bodies() {
        let bodies = read_main_bodies().unwrap();
        assert_eq!(bodies.len(), 366);
    }

    #[test]
    fn test_fix_bodies() {
        let bodies = read_main_bodies().unwrap();
        let sun = bodies.iter().find(|data| data.id == SUN_ID.into()).unwrap();
        assert!(sun.host_body.is_none());
        for planet in bodies
            .iter()
            .filter(|data| matches!(data.body_type, BodyType::Planet))
        {
            assert!(planet.host_body.is_some_and(|id| id == SUN_ID.into()))
        }
    }
}
