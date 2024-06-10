use body::Body;
use body_data::BodyType;
use body_id::BodyID;
use std::{cell::RefCell, io::Result, rc::Rc};

use crate::utils::de::read_main_bodies;

use self::body_data::BodyData;

pub mod body;
pub mod body_data;
pub mod body_id;

#[derive(Default)]
pub struct BodySystem<'a> {
    pub main_body: Option<Body<'a>>,
    pub bodies: Vec<&'a Body<'a>>,
    pub time: f64,
}

impl BodySystem {
    pub fn simple_solar_system() -> Result<Self> {
        let mut all_data: Vec<BodyData> = read_main_bodies()?
            .into_iter()
            .filter(|data| matches!(data.body_type, BodyType::Planet | BodyType::Star))
            .collect();
        all_data.sort_by(|a, b| a.periapsis.cmp(&b.periapsis));
        let mut sun = all_data[0];
        sun.orbiting_bodies = all_data[1..].iter().map(|planet| planet.id).collect();
        all_data[1..]
            .iter_mut()
            .for_each(|planet| planet.host_body = sun.id);

        let system = Rc::new(RefCell::new(BodySystem::default()));
        let mut sun = Body::new_loner(sun, Rc::clone(&system));

        Ok(Self {
            bodies: all_data.into_iter().map(|data| data.into()).collect(),
            time: 0.,
        })
    }

    pub fn get_body_data(&self, id: &BodyID) -> Option<&Body> {
        self.bodies.iter().find(|body| body.data.id == *id)
    }

    pub fn get_body_names(&self) -> Vec<&str> {
        self.bodies.iter().map(|b| &b.info.name[..]).collect()
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
