use body::Body;
use body_data::BodyType;
use body_id::BodyID;
use std::io::Result;

use crate::utils::de::read_main_bodies;

use self::body_data::BodyData;

pub mod body;
pub mod body_data;
pub mod body_id;

pub struct BodySystem {
    pub bodies: Vec<Body>,
    time: f64,
}

impl BodySystem {
    pub fn simple_solar_system() -> Result<Self> {
        let mut all_data: Vec<BodyData> = read_main_bodies()?
            .into_iter()
            .filter(|data| matches!(data.body_type, BodyType::Planet | BodyType::Star))
            .collect();
        all_data.sort_by(|a, b| a.periapsis.cmp(&b.periapsis));

        Ok(Self {
            bodies: all_data.into_iter().map(|data| data.into()).collect(),
            time: 0.,
        })
    }

    pub fn get_body_data(&self, id: &BodyID) -> Option<&Body> {
        self.bodies.iter().find(|body| body.data.id == *id)
    }

    pub fn get_body_names(&self) -> Vec<&str> {
        self.bodies.iter().map(|b| &b.data.name[..]).collect()
    }

    pub fn number(&self) -> usize {
        self.bodies.len()
    }

    pub fn get_max_distance(&self) -> i64 {
        self.bodies
            .iter()
            .map(|body| body.data.apoapsis)
            .max()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::BodySystem;

    #[test]
    fn test_simple_solar_system() {
        let system = BodySystem::simple_solar_system().unwrap();
        assert_eq!(system.bodies.len(), 9)
    }

    #[test]
    fn test_max_distance() {
        let system = BodySystem::simple_solar_system().unwrap();
        assert_eq!(system.get_max_distance(), 4537039826)
    }
}
