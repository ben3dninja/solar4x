use std::time::Duration;

use bevy::{math::DVec3, prelude::*};

use crate::{
    bodies::body_id::BodyID,
    core_plugin::{build_system, BodiesMapping, BodyInfo, LoadingState, PrimaryBody},
    engine_plugin::{Position, Velocity},
    main_game::GameStage,
    GAMETIME_PER_SIMTICK, SECONDS_PER_DAY, TPS,
};

/// Gravitationnal constant in km3kg-1d-2
pub const G: f64 = 6.6743e-11 * SECONDS_PER_DAY * SECONDS_PER_DAY * 1e-9;

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TickTimer(Timer::new(
            Duration::from_secs_f64(1. / TPS),
            TimerMode::Repeating,
        )))
        .add_systems(
            OnEnter(LoadingState::Loading),
            setup_hill_spheres.after(build_system),
        )
        .add_systems(
            FixedUpdate,
            (update_influence, apply_gravity_force, integrate_positions)
                .chain()
                .run_if(in_state(GameStage::Action)),
        );
    }
}

#[derive(Resource)]
pub struct TickTimer(pub Timer);

#[derive(Component)]
pub struct Mass(pub f64);

#[derive(Component)]
pub struct HillRadius(f64);

/// Component storing the bodies that influence the object's trajectory
#[derive(Component, Default, Debug)]
pub struct Influenced {
    pub main_influencer: Option<Entity>,
    pub influencers: Vec<Entity>,
}

pub fn setup_hill_spheres(
    mut commands: Commands,
    query: Query<&BodyInfo>,
    primary: Query<(Entity, &BodyInfo), With<PrimaryBody>>,
    mapping: Res<BodiesMapping>,
) {
    let mut queue = vec![(primary.single().1 .0.id, 0.)];
    let mut i = 0;
    while i < queue.len() {
        let (id, parent_mass) = queue[i];
        if let Some(entity) = mapping.0.get(&id) {
            if let Ok(BodyInfo(data)) = query.get(*entity) {
                let radius = (data.semimajor_axis
                    * (1. - data.eccentricity)
                    * (data.mass / (3. * (parent_mass + data.mass))).powf(1. / 3.))
                .max(data.radius);
                commands.entity(*entity).insert(HillRadius(radius));
                queue.extend(data.orbiting_bodies.iter().map(|c| (*c, data.mass)));
            }
        }
        i += 1;
    }
    commands
        .entity(primary.single().0)
        .insert(HillRadius(f64::INFINITY));
}

pub fn update_influence(
    mut influenced: Query<(&Position, &mut Influenced)>,
    bodies: Query<(&Position, &Mass, &HillRadius, &BodyInfo)>,
    mapping: Res<BodiesMapping>,
    main_body: Query<&BodyInfo, With<PrimaryBody>>,
    timer: Res<TickTimer>,
) {
    if timer.0.just_finished() {
        let main_body = main_body.single().0.id;
        influenced
            .par_iter_mut()
            .for_each(|(object_pos, mut influence)| {
                *influence = compute_influence(object_pos, &bodies, mapping.as_ref(), main_body);
            });
    }
}

pub fn compute_influence(
    Position(object_pos): &Position,
    bodies: &Query<(&Position, &Mass, &HillRadius, &BodyInfo)>,
    mapping: &BodiesMapping,
    main_body: BodyID,
) -> Influenced {
    // if an object is not in a bodie's sphere of influence, it is not in its children's either
    fn influencers_rec(
        body: BodyID,
        query: &Query<(&Position, &Mass, &HillRadius, &BodyInfo)>,
        mapping: &BodiesMapping,
        object_pos: &DVec3,
        influences: &mut Vec<(Entity, f64)>,
    ) {
        if let Some(e) = mapping.0.get(&body) {
            let (Position(body_pos), Mass(mass), HillRadius(hill_radius), BodyInfo(data)) =
                query.get(*e).unwrap();
            let r = *object_pos - *body_pos;
            let dist = r.length();
            if dist < *hill_radius {
                influences.push((*e, *mass / (dist * dist)));
                data.orbiting_bodies.iter().for_each(|child| {
                    influencers_rec(*child, query, mapping, object_pos, influences);
                })
            }
        }
    }

    let mut influences = Vec::new();
    influencers_rec(main_body, bodies, mapping, object_pos, &mut influences);
    Influenced {
        main_influencer: influences
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|a| a.0),
        influencers: influences.into_iter().map(|a| a.0).collect(),
    }
}

#[derive(Component, Debug, Default)]
pub struct Acceleration(pub DVec3);

pub fn apply_gravity_force(
    mut gravity_bound: Query<(&Position, &mut Acceleration, &Influenced)>,
    bodies: Query<(&Position, &Mass)>,
) {
    gravity_bound
        .par_iter_mut()
        .for_each(|(object_pos, mut acceleration, influenced)| {
            acceleration.0 = compute_acceleration(
                object_pos.0,
                bodies
                    .iter_many(&influenced.influencers)
                    .map(|(p, m)| (p.0, m.0)),
            );
        });
}

/// Computes the acceleration from the object's position, and an iterator of the influencers' positions and masses
pub fn compute_acceleration(
    object_pos: DVec3,
    influencers: impl Iterator<Item = (DVec3, f64)>,
) -> DVec3 {
    let mut acc = DVec3::ZERO;
    for (body_pos, mass) in influencers {
        let r = object_pos - body_pos;
        let dist = r.length();
        acc -= r * G * mass / (dist.powi(3));
    }
    acc
}

pub fn integrate_positions(mut query: Query<(&mut Position, &mut Velocity, &Acceleration)>) {
    // Gametime days since last update
    let dt = GAMETIME_PER_SIMTICK;
    query.par_iter_mut().for_each(|(mut pos, mut speed, acc)| {
        let (dr, dv) = compute_deltas(speed.0, acc.0, dt);
        pos.0 += dr;
        speed.0 += dv;
    });
}

/// Returns delta v and delta r for the following integration step
pub fn compute_deltas(speed: DVec3, acc: DVec3, dt: f64) -> (DVec3, DVec3) {
    let dv = acc * dt;
    let dr = (speed + dv) * dt;
    (dv, dr)
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        bodies::{bodies_config::BodiesConfig, body_data::BodyType, body_id::id_from},
        client_plugin::{ClientMode, ClientPlugin},
        core_plugin::BodiesMapping,
        engine_plugin::{Position, Velocity},
        main_game::ShipEvent,
        spaceship::ShipInfo,
        utils::algebra::circular_orbit_around_body,
    };

    use super::{Influenced, Mass};

    #[test]
    fn test_influence() {
        let mut app = App::new();
        app.add_plugins(
            ClientPlugin::testing()
                .with_bodies(BodiesConfig::SmallestBodyType(BodyType::Moon))
                .in_mode(ClientMode::Singleplayer),
        );
        app.update();
        let world = app.world_mut();
        let mapping = &world.resource::<BodiesMapping>().0;
        let moon = mapping[&id_from("lune")];
        let earth = mapping[&id_from("terre")];
        let sun = mapping[&id_from("soleil")];
        let (mass, pos, speed) = world
            .query::<(&Mass, &Position, &Velocity)>()
            .get(world, moon)
            .unwrap();
        let (spawn_pos, spawn_speed) = circular_orbit_around_body(100., mass.0, pos.0, speed.0);
        dbg!(pos, speed);
        dbg!(spawn_pos, spawn_speed);
        world.send_event(ShipEvent::Create(ShipInfo {
            id: id_from("s"),
            spawn_pos,
            spawn_speed,
        }));
        app.update();
        let world = app.world_mut();
        let influenced = world.query::<&Influenced>().single(world);

        assert!(influenced.influencers.contains(&moon));
        assert!(influenced.influencers.contains(&earth));
        assert!(influenced.influencers.contains(&sun));
        assert_eq!(influenced.main_influencer, Some(moon));
        assert_eq!(influenced.influencers.len(), 3);
    }
}
