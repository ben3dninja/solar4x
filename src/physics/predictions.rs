use std::collections::BTreeMap;

use bevy::{math::DVec3, prelude::*, utils::HashMap};

use crate::{
    objects::{prelude::*, ships::trajectory::ManeuverNode},
    physics::prelude::*,
};

use super::{
    leapfrog::{get_acceleration, get_dv, get_dx},
    time::GAMETIME_PER_SIMTICK,
};

/// Number of client updates between two predictions
pub const PREDICTIONS_STEP: usize = 20;

/// A component representing identifying a prediction of a ship at a selected time
#[derive(Component, Clone, Copy)]
pub struct Prediction {
    pub ship: Entity,
    pub index: usize,
    pub simtick: u64,
}

#[derive(Debug)]
pub struct PredictionStart {
    pub pos: DVec3,
    pub speed: DVec3,
    pub acc: DVec3,
    pub tick: u64,
}

impl PredictionStart {
    /// Compute the future positions of this point with respect to a given referential and considering some influencer's gravitationnal pull on it
    pub fn compute_predictions(
        &self,
        number: usize,
        influencers: impl Iterator<Item = Entity> + Clone,
        reference: Option<Entity>,
        bodies: &Query<(&EllipticalOrbit, &BodyInfo)>,
        mapping: &HashMap<BodyID, Entity>,
        nodes: &BTreeMap<u64, ManeuverNode>,
    ) -> Vec<(DVec3, DVec3)> {
        let dt = GAMETIME_PER_SIMTICK;
        let mut pos = self.pos;
        let mut speed = self.speed;
        let mut predictions = Vec::new();
        let ref_index = reference.and_then(|r| influencers.clone().position(|e| e == r));
        let masses = influencers
            .clone()
            .map(|e| bodies.get(e).unwrap().1 .0.mass);
        let initial_bodies_coords =
            get_bodies_coordinates(influencers.clone(), bodies, mapping, self.tick);
        let initial_ref_coords =
            ref_index.map_or((DVec3::ZERO, DVec3::ZERO), |i| initial_bodies_coords[i]);
        let mut acc = self.acc;
        let mut previous_acc;
        for i in 1..number + 1 {
            let tick = self.tick + i as u64;
            if let Some(node) = nodes.get(&tick) {
                speed += node.thrust;
            }
            pos += get_dx(speed, acc, dt);
            let (bodies_pos, bodies_speeds): (Vec<_>, Vec<_>) =
                get_bodies_coordinates(influencers.clone(), bodies, mapping, tick)
                    .into_iter()
                    .unzip();
            let ref_coords = ref_index.map_or((DVec3::ZERO, DVec3::ZERO), |i| {
                (bodies_pos[i], bodies_speeds[i])
            });
            previous_acc = acc;
            acc = get_acceleration(pos, bodies_pos.into_iter().zip(masses.clone()));
            speed += get_dv(previous_acc, acc, dt);
            predictions.push((
                pos - ref_coords.0 + initial_ref_coords.0,
                speed - ref_coords.1,
            ));
        }
        predictions
    }
}

pub fn get_bodies_coordinates(
    selected_bodies: impl Iterator<Item = Entity>,
    bodies: &Query<(&EllipticalOrbit, &BodyInfo)>,
    mapping: &HashMap<BodyID, Entity>,
    tick: u64,
) -> Vec<(DVec3, DVec3)> {
    fn compute_pos_rec(
        e: Entity,
        map: &mut HashMap<Entity, (DVec3, DVec3)>,
        tick: u64,
        bodies: &Query<(&EllipticalOrbit, &BodyInfo)>,
        mapping: &HashMap<BodyID, Entity>,
    ) -> (DVec3, DVec3) {
        let (orbit, BodyInfo(data)) = bodies.get(e).unwrap();
        let mut o = orbit.clone();
        o.update_pos(tick as f64 * GAMETIME_PER_SIMTICK);
        if let Some(coords) = map.get(&e) {
            *coords
        } else {
            let (pos, speed) = data.host_body.map_or((DVec3::ZERO, DVec3::ZERO), |parent| {
                let (parent_pos, parent_speed) =
                    compute_pos_rec(mapping[&parent], map, tick, bodies, mapping);
                (parent_pos + o.local_pos, parent_speed + o.local_speed)
            });
            map.insert(e, (pos, speed));
            (pos, speed)
        }
    }

    let mut map = HashMap::new();

    selected_bodies
        .map(|e| compute_pos_rec(e, &mut map, tick, bodies, mapping))
        .collect()
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::system::SystemState, prelude::*};

    use crate::{
        physics::leapfrog::get_acceleration, prelude::*, utils::algebra::circular_orbit_around_body,
    };

    use super::*;

    #[test]
    fn test_predictions() {
        let mut app = App::new();
        app.add_plugins(ClientPlugin::testing().in_mode(ClientMode::Singleplayer));
        app.update();
        let world = app.world_mut();
        let mapping = &world.resource::<BodiesMapping>().0;
        let earth = *mapping.get(&id_from("terre")).unwrap();
        let sun = *mapping.get(&id_from("soleil")).unwrap();
        let (&mass, &earth_pos, &earth_speed) = world
            .query::<(&Mass, &Position, &Velocity)>()
            .get(world, earth)
            .unwrap();
        let (pos, speed) = circular_orbit_around_body(1e5, mass.0, earth_pos.0, earth_speed.0);
        let influencers = vec![sun, earth];
        #[allow(clippy::type_complexity)]
        let mut system_state: SystemState<(
            Res<BodiesMapping>,
            Query<(&EllipticalOrbit, &BodyInfo)>,
            Query<(&Position, &Mass)>,
        )> = SystemState::new(world);
        let (mapping, bodies, query) = system_state.get(world);
        let predictions = PredictionStart {
            pos,
            speed,
            tick: 0,
            acc: get_acceleration(pos, query.iter_many(&influencers).map(|(p, m)| (p.0, m.0))),
        }
        .compute_predictions(
            3,
            influencers.into_iter(),
            Some(earth),
            &bodies,
            &mapping.0,
            &BTreeMap::new(),
        );
        for (i, (p, _)) in predictions.into_iter().enumerate() {
            // dbg!(p);
            // dbg!(pos + (i + 1) as f64 * (speed - earth_speed.0) * GAMETIME_PER_SIMTICK);
            assert!(
                (p - (pos + (i + 1) as f64 * (speed - earth_speed.0) * GAMETIME_PER_SIMTICK))
                    .length()
                    <= 5e4
            );
        }
    }
}
