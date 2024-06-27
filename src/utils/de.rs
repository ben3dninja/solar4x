use std::{fs::File, io::Read};

use serde::{Deserialize, Deserializer};

use crate::bodies::body_data::BodyData;

const MAIN_OBJECT_FILE_PATH: &str = "main_objects.json";
const SUN_ID: &str = "soleil";

pub fn deserialize_options<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Deserialize::deserialize(d).map(|x: Option<T>| x.unwrap_or_default())
}

pub fn read_main_bodies() -> std::io::Result<Vec<BodyData>> {
    let mut file = File::open(MAIN_OBJECT_FILE_PATH)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    #[derive(Deserialize)]
    struct Input {
        bodies: Vec<BodyData>,
    }
    let input: Input = serde_json::from_str(&buf).map_err(std::io::Error::from)?;
    fix_bodies(input.bodies)
}

fn fix_bodies(mut bodies: Vec<BodyData>) -> std::io::Result<Vec<BodyData>> {
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
