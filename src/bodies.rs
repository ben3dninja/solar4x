use body::Body;
use body_data::BodyType;
use body_id::BodyID;
use std::{cell::RefCell, collections::HashMap, io::Result, mem, rc::Rc};

use crate::utils::de::read_main_bodies;

use self::{
    body::{BodyInfo, UpdateState},
    body_data::BodyData,
};

pub mod body;
pub mod body_data;
pub mod body_id;

#[derive(Default)]
pub struct BodySystem {
    pub bodies: HashMap<BodyID, Body>,
    pub time: f64,
}

impl BodySystem {
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
        {
            let mut mut_system = system.borrow_mut();
            mut_system.bodies.insert(
                sun_data.id.clone(),
                Body::new_loner(&sun_data, Rc::clone(&system)),
            );

            let planets = planets_data.iter().map(|data| {
                (
                    data.id.clone(),
                    Body::new_loner(data, Rc::clone(&system)).with_host_body(sun_data.id.clone()),
                )
            });
            mut_system.bodies.extend(planets.into_iter());
        }
        Ok(system)
    }

    pub fn number(&self) -> usize {
        self.bodies.len()
    }

    pub fn get_max_distance(&self) -> i64 {
        self.bodies
            .values()
            .map(|body| body.info.apoapsis)
            .max()
            .unwrap()
    }

    pub fn bodies_by_distance(&self) -> Vec<BodyID> {
        let mut list: Vec<&Body> = self.bodies.values().collect();
        list.sort_by(|a, b| a.info.periapsis.cmp(&b.info.periapsis));
        list.iter().map(|body| body.id.clone()).collect()
    }

    pub fn update_orbits(&mut self) {
        self.bodies
            .values_mut()
            .for_each(|body| body.orbit.update_pos(self.time))
    }

    pub fn set_time(&mut self, new_time: f64) {
        self.time = new_time;
        self.bodies
            .values_mut()
            .for_each(|body| body.orbit.update_state = UpdateState::None)
    }

    pub fn elapse_time(&mut self, dt: f64) {
        self.set_time(self.time + dt)
    }
}

#[cfg(test)]
mod tests {
    use crate::bodies::body_id::BodyID;

    use super::BodySystem;

    #[test]
    fn test_simple_solar_system() {
        let system = BodySystem::simple_solar_system().unwrap();
        assert_eq!(system.take().bodies.len(), 9)
    }

    #[test]
    fn test_max_distance() {
        let system = BodySystem::simple_solar_system().unwrap();
        assert_eq!(system.take().get_max_distance(), 4537039826)
    }

    #[test]
    fn test_bodies_by_distance() {
        let system = BodySystem::simple_solar_system().unwrap();
        assert_eq!(
            system.take().bodies_by_distance(),
            vec![
                "soleil", "mercure", "venus", "terre", "mars", "jupiter", "saturne", "uranus",
                "neptune"
            ]
            .into_iter()
            .map(Into::<BodyID>::into)
            .collect::<Vec<_>>()
        )
    }
}
