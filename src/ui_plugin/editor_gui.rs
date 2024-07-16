use bevy::{math::DVec3, prelude::*, utils::HashMap};

use crate::{
    bodies::body_id::BodyID,
    core_plugin::{BodiesMapping, BodyInfo},
    engine_plugin::{EllipticalOrbit, GameTime, Position},
    gravity::{compute_acceleration, compute_deltas, Influenced},
    GAMETIME_PER_SIMTICK,
};

use super::{
    editor_screen::{EditorContext, InEditor},
    CreateScreen,
};

/// Number of client updates between two predictions
const PREDICTIONS_STEP: usize = 3;
const PREDICTIONS_NUMBER: usize = 100;

pub struct EditorGuiPlugin;

impl Plugin for EditorGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InEditor), create_predictions.after(CreateScreen));
    }
}

/// A component representing identifying a prediction of a ship at a selected time
#[derive(Component)]
pub struct Prediction {
    pub ship: Entity,
    pub index: usize,
}

fn create_predictions(
    mut commands: Commands,
    ctx: Res<EditorContext>,
    influence: Query<&Influenced>,
    bodies: Query<(&EllipticalOrbit, &BodyInfo)>,
    bodies_mapping: Res<BodiesMapping>,
    time: Res<GameTime>,
) {
    let start = SpaceTimePoint {
        pos: ctx.pos,
        speed: ctx.speed,
        time: time.time(),
    };
    let predictions = start.compute_predictions(
        PREDICTIONS_NUMBER,
        influence.get(ctx.ship).unwrap().influencers.iter().cloned(),
        &bodies,
        &bodies_mapping.0,
    );
    eprintln!("{start:?}, {predictions:?}");
    predictions.into_iter().enumerate().for_each(|(i, p)| {
        commands.spawn((
            Prediction {
                ship: ctx.ship,
                index: i,
            },
            Position(p),
            TransformBundle::from_transform(Transform::from_xyz(0., 0., -3.)),
        ));
    });
}

#[derive(Debug)]
struct SpaceTimePoint {
    pos: DVec3,
    speed: DVec3,
    time: f64,
}

impl SpaceTimePoint {
    fn compute_predictions(
        &self,
        number: usize,
        influencers: impl Iterator<Item = Entity> + Clone,
        bodies: &Query<(&EllipticalOrbit, &BodyInfo)>,
        mapping: &HashMap<BodyID, Entity>,
    ) -> Vec<DVec3> {
        let mut pos = self.pos;
        let mut speed = self.speed;
        let mut predictions = Vec::new();
        let masses = influencers
            .clone()
            .map(|e| bodies.get(e).unwrap().1 .0.mass);
        for i in 0..number {
            let positions = get_bodies_pos(
                influencers.clone(),
                bodies,
                mapping,
                self.time + (i * PREDICTIONS_STEP) as f64 * GAMETIME_PER_SIMTICK,
            );
            let acc = compute_acceleration(pos, positions.into_iter().zip(masses.clone()));
            let (dv, dr) =
                compute_deltas(speed, acc, GAMETIME_PER_SIMTICK * PREDICTIONS_STEP as f64);
            speed += dv;
            pos += dr;
            predictions.push(pos);
        }
        predictions
    }
}

fn get_bodies_pos(
    selected_bodies: impl Iterator<Item = Entity>,
    bodies: &Query<(&EllipticalOrbit, &BodyInfo)>,
    mapping: &HashMap<BodyID, Entity>,
    time: f64,
) -> Vec<DVec3> {
    fn compute_pos_rec(
        e: Entity,
        map: &mut HashMap<Entity, DVec3>,
        time: f64,
        bodies: &Query<(&EllipticalOrbit, &BodyInfo)>,
        mapping: &HashMap<BodyID, Entity>,
    ) -> DVec3 {
        let (orbit, BodyInfo(data)) = bodies.get(e).unwrap();
        let mut o = orbit.clone();
        o.update_pos(time);
        if let Some(pos) = map.get(&e) {
            *pos
        } else {
            let pos = data.host_body.map_or(DVec3::ZERO, |parent| {
                compute_pos_rec(mapping[&parent], map, time, bodies, mapping) + o.local_pos
            });
            map.insert(e, pos);
            pos
        }
    }

    let mut map = HashMap::new();

    selected_bodies
        .map(|e| compute_pos_rec(e, &mut map, time, bodies, mapping))
        .collect()
}

#[cfg(test)]
mod tests {
    use bevy::{
        app::App,
        ecs::system::{Query, Res, SystemState},
    };

    use crate::{
        bodies::body_id::id_from,
        client_plugin::{ClientMode, ClientPlugin},
        core_plugin::{BodiesMapping, BodyInfo},
        engine_plugin::{EllipticalOrbit, Position, Velocity},
        gravity::Mass,
        utils::algebra::circular_orbit_around_body,
        GAMETIME_PER_SIMTICK,
    };

    use super::SpaceTimePoint;

    #[test]
    fn test_predictions() {
        let mut app = App::new();
        app.add_plugins(ClientPlugin::testing().in_mode(ClientMode::Singleplayer));
        app.update();
        let world = app.world_mut();
        let mapping = &world.resource::<BodiesMapping>().0;
        let earth = *mapping.get(&id_from("terre")).unwrap();
        let sun = *mapping.get(&id_from("soleil")).unwrap();
        let (mass, earth_pos, earth_speed) = world
            .query::<(&Mass, &Position, &Velocity)>()
            .get(world, earth)
            .unwrap();
        let (pos, speed) = circular_orbit_around_body(1e4, mass.0, earth_pos.0, earth_speed.0);
        let influencers = vec![sun, earth];
        #[allow(clippy::type_complexity)]
        let mut system_state: SystemState<(
            Res<BodiesMapping>,
            Query<(&EllipticalOrbit, &BodyInfo)>,
        )> = SystemState::new(world);
        let (mapping, bodies) = system_state.get(world);
        let predictions = SpaceTimePoint {
            pos,
            speed,
            time: 0.,
        }
        .compute_predictions(3, influencers.into_iter(), &bodies, &mapping.0);
        for (i, p) in predictions.into_iter().enumerate() {
            assert!((p - (pos + (i + 1) as f64 * speed * GAMETIME_PER_SIMTICK)).length() <= 1e5);
        }
    }
}
