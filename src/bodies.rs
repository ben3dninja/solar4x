use body::Body;
use body_data::BodyType;
use body_id::BodyID;
use std::{cell::RefCell, collections::HashMap, io::Result, rc::Rc};

use crate::utils::de::read_main_bodies;

use self::{body::UpdateState, body_data::BodyData};

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
        Self::new_system_with_filter(|data| {
            matches!(data.body_type, BodyType::Star | BodyType::Planet)
        })
    }

    pub fn new_system_with_filter(f: impl FnMut(&BodyData) -> bool) -> Result<Rc<RefCell<Self>>> {
        let all_data: Vec<BodyData> = read_main_bodies()?.into_iter().filter(f).collect();
        let system = Rc::new(RefCell::new(BodySystem::default()));
        let bodies: HashMap<BodyID, Body> = all_data
            .into_iter()
            .map(|data| (data.id.clone(), Body::new(data, Rc::clone(&system))))
            .collect();
        system.borrow_mut().bodies = bodies;
        Ok(system)
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

    pub fn primary_body_id(&self) -> Option<BodyID> {
        self.bodies
            .values()
            .find(|body| body.host_body.is_none())
            .map(|body| body.id.clone())
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

    #[test]
    fn test_primary_body() {
        let system = BodySystem::simple_solar_system().unwrap();
        assert_eq!(system.take().primary_body_id().unwrap(), "soleil".into())
    }
}
