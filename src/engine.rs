mod body;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use nalgebra::Vector3;

use crate::app::{body_id::BodyID, info::SystemInfo, GlobalMap, TIME_STEP};

use self::body::{Body, UpdateState};

// Speed in days per second
const DEFAULT_SPEED: f64 = 10.;
// World origin (coordinates of primary body)
const ORIGIN: Vector3<i64> = Vector3::new(0, 0, 0);

pub struct Engine {
    pub bodies: HashMap<BodyID, Body>,
    // 1 represents 1 day / second
    pub speed: f64,
    pub time: f64,
    pub global_map: Arc<Mutex<GlobalMap>>,
    shared_info: Arc<SystemInfo>,
}

impl Engine {
    pub fn new_from_data(global_map: Arc<Mutex<GlobalMap>>, shared_info: Arc<SystemInfo>) -> Self {
        let bodies: HashMap<BodyID, Body> = shared_info
            .bodies
            .iter()
            .map(|(id, data)| (*id, data.into()))
            .collect();
        Self {
            bodies,
            speed: DEFAULT_SPEED,
            global_map,
            shared_info,
            time: 0.,
        }
    }

    pub fn update(&mut self) {
        self.time += self.speed * TIME_STEP.as_secs_f64();
        self.update_local();
        self.update_global();
    }

    fn update_local(&mut self) {
        // TODO : parallelize with rayon
        self.bodies.values_mut().for_each(|body| {
            body.update_state = UpdateState::None;
            body.update_pos(self.time)
        });
    }

    fn update_global(&self) {
        let id = self.shared_info.primary_body;
        let mut buffer = self.global_map.lock().unwrap();
        self._update_global_rec(&mut buffer, id);
    }

    fn _update_global_rec(&self, buffer: &mut MutexGuard<'_, GlobalMap>, id: BodyID) {
        if let Some(body) = self.bodies.get(&id) {
            if let Some(data) = self.shared_info.bodies.get(&id) {
                let host_position = *data
                    .host_body
                    .and_then(|host_id| buffer.get(&host_id))
                    .unwrap_or(&ORIGIN);
                buffer.insert(id, body.position + host_position);
                for child in &data.orbiting_bodies {
                    self._update_global_rec(buffer, *child);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        app::{body_id::BodyID, App, TIME_STEP},
        utils::algebra::inorm,
    };

    #[test]
    fn test_update_global() {
        let mut app = App::new_moons(true).unwrap();
        app.engine.update();
        let global = app.next_map.lock().unwrap();
        let local = &app.engine.bodies;
        let moon = "lune".into();
        assert!(
            (inorm(global[&moon]) - inorm(local[&"terre".into()].position)).abs()
                <= inorm(local[&moon].position)
        )
    }

    #[test]
    fn test_speed() {
        let mut app = App::new_moons(true).unwrap();
        app.engine.speed = 10.;
        let mut time = 0.;
        let moon = BodyID::from("lune");
        app.engine.update();
        let initial_pos = app.engine.bodies[&moon].position;

        let period = app.shared_info.bodies[&moon].revolution_period;
        while time < period {
            time += 10. * TIME_STEP.as_secs_f64();
            app.engine.update();
        }
        let final_pos = app.engine.bodies[&moon].position;
        dbg!(final_pos, initial_pos);
        assert!(inorm(final_pos - initial_pos) < 5000)
    }
}
