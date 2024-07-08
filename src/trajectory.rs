use std::sync::Arc;

use bevy::{math::DVec3, prelude::*};
use std::sync::Mutex;

use crate::{
    bodies::body_id::BodyID,
    core_plugin::EntityMapping,
    engine_plugin::{GameTime, Position, Velocity},
    spaceship::{ShipID, ShipInfo},
    utils::algebra::convert_orbital_to_global,
};

#[derive(Component)]
pub struct Trajectory {
    pub nodes: Vec<ManeuverNode>,
    pub current: usize,
}
pub struct ManeuverNode {
    pub name: String,
    pub time: f64,
    pub thrust: DVec3,
    pub origin: BodyID,
}

#[derive(Event, Debug)]
pub struct VelocityUpdate {
    pub ship_id: ShipID,
    pub thrust: DVec3,
}
// impl VelocityUpdate{
//     pub fn from_node(ship_id: ShipID, node: &ManeuverNode, )
// }

pub fn follow_trajectory(
    mut velocity_events: EventWriter<VelocityUpdate>,
    mapping: Res<EntityMapping>,
    coords: Query<(&Position, &Velocity)>,
    mut trajectories: Query<(Entity, &mut Trajectory, &ShipInfo)>,
    time: Res<GameTime>,
) {
    let events = Arc::new(Mutex::new(Vec::new()));
    trajectories.par_iter_mut().for_each(|(e, mut t, info)| {
        if let Some(n) = t.nodes.get(t.current + 1) {
            if n.time >= time.0 {
                if let Some(origin) = mapping.id_mapping.get(&n.origin) {
                    let (&Position(o_pos), &Velocity(o_speed)) = coords.get(*origin).unwrap();
                    let (&Position(pos), &Velocity(speed)) = coords.get(e).unwrap();
                    let thrust = convert_orbital_to_global(n.thrust, o_pos, o_speed, pos, speed);
                    events.lock().unwrap().push(VelocityUpdate {
                        ship_id: info.id,
                        thrust,
                    });
                }
            }
            t.current += 1;
        }
    });
    velocity_events.send_batch(Arc::try_unwrap(events).unwrap().into_inner().unwrap());
}
