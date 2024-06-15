use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use nalgebra::Vector3;

use crate::{
    app::{GlobalMap, SystemInfo, TIME_STEP},
    bodies::{body::Body, body_id::BodyID},
};

// Speed in days per second
const DEFAULT_SPEED: f64 = 10.;
// World origin (coordinates of primary body)
const ORIGIN: Vector3<i64> = Vector3::new(0, 0, 0);

pub struct Engine {
    pub bodies: HashMap<BodyID, Body>,
    // 1 represents 1 day / second
    pub speed: f64,
    pub time: f64,
    global_map: Arc<Mutex<GlobalMap>>,
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
        self.time += self.speed / TIME_STEP.as_secs_f64();
        self.update_local();
    }

    fn update_local(&mut self) {
        // TODO : parallelize with rayon
        self.bodies
            .values_mut()
            .for_each(|body| body.update_pos(self.time));
    }

    fn update_global(&self) {
        let id = self.shared_info.primary_body;
        let mut buffer = self.global_map.lock().unwrap();
        self._update_global_rec(&mut buffer, id);
    }

    fn _update_global_rec(&self, buffer: &mut MutexGuard<'_, GlobalMap>, id: BodyID) {
        if let Some(body) = self.bodies.get(&id) {
            if let Some(data) = self.shared_info.bodies.get(&id) {
                let host_position = data
                    .host_body
                    .and_then(|host_id| buffer.get(&host_id))
                    .unwrap_or(&ORIGIN)
                    .clone();
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
    use crate::app::App;

    #[test]
    fn test_update_global() {
        let app = App::new_moons(true);
    }
}
