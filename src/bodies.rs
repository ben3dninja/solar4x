use body::Body;
use body_data::BodyType;
use body_id::BodyID;
use std::{cell::RefCell, io::Result, mem, rc::Rc};

use crate::utils::de::read_main_bodies;

use self::{body::BodyInfo, body_data::BodyData};

pub mod body;
pub mod body_data;
pub mod body_id;

#[derive(Default)]
pub struct BodySystem<'a> {
    pub bodies: Vec<Body<'a>>,
    pub time: f64,
}

impl<'a> BodySystem<'a> {
    pub fn simple_solar_system() -> Result<Rc<RefCell<Self>>> {
        let mut all_data: Vec<BodyData> = read_main_bodies()?
            .into_iter()
            .filter(|data| matches!(data.body_type, BodyType::Planet | BodyType::Star))
            .collect();
        all_data.sort_by(|a, b| a.periapsis.cmp(&b.periapsis));
        let planets_data = all_data.split_off(1);
        let mut sun_data = mem::take(&mut all_data[0]);
        sun_data.orbiting_bodies = planets_data
            .iter()
            .map(|planet| planet.id.clone())
            .collect();
        all_data[1..]
            .iter_mut()
            .for_each(|planet| planet.host_body = sun_data.id.clone());

        let system = Rc::new(RefCell::new(BodySystem::default()));
        system
            .borrow_mut()
            .bodies
            .push(Body::new_loner(&sun_data, Rc::clone(&system)));

        let planets = planets_data
            .iter()
            .map(|data| Body::new_loner(data, Rc::clone(&system)));
        let mut bodies = vec![Body::new_loner(&sun_data, Rc::clone(&system))];
        bodies.extend(planets);
        system.borrow_mut().bodies = bodies;
        let mut ref_system = system.borrow_mut();
        ref_system.bodies[1..]
            .iter_mut()
            .map(|body| body.host_body = Some(&ref_system.bodies[0]));

        Ok(system)
    }

    pub fn get_body_data(&self, id: &BodyID) -> Option<BodyInfo> {
        self.bodies
            .iter()
            .find(|body| body.id == *id)
            .map(|body| body.info.clone())
    }

    pub fn get_body_names(&self) -> Vec<String> {
        self.bodies
            .iter()
            .map(|b| b.borrow().info.name.clone())
            .collect()
    }

    pub fn number(&self) -> usize {
        self.bodies.len()
    }

    pub fn get_max_distance(&self) -> i64 {
        self.bodies
            .iter()
            .map(|body| body.info.apoapsis)
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
