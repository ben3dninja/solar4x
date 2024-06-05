use body::Body;
use body_data::BodyType;
use body_id::BodyID;
use std::io::Result;

use crate::utils::de::read_main_bodies;

pub mod body;
pub mod body_data;
pub mod body_id;

pub struct BodySystem {
    bodies: Vec<Body>,
}

impl BodySystem {
    pub fn simple_solar_system() -> Result<Self> {
        Ok(Self {
            bodies: read_main_bodies()?
                .into_iter()
                .filter(|data| matches!(data.body_type, BodyType::Planet | BodyType::Star))
                .map(|data| data.into())
                .collect(),
        })
    }

    pub fn get_body_data(&self, id: &BodyID) -> Option<&Body> {
        self.bodies.iter().find(|body| body.data.id == *id)
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
}
