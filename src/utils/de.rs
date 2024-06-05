use std::{fs::File, io::Read};

use serde::{Deserialize, Deserializer};

use crate::bodies::body_data::BodyData;

const MAIN_OBJECT_FILE_PATH: &'static str = "main_objects.json";

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
    let input: Input = serde_json::from_str(&buf).map_err(|e| std::io::Error::from(e))?;
    Ok(input.bodies)
}

#[cfg(test)]
mod tests {
    use super::read_main_bodies;

    #[test]
    fn test_read_main_bodies() -> std::io::Result<()> {
        let bodies = read_main_bodies()?;
        assert_eq!(bodies.len(), 366);
        Ok(())
    }
}
