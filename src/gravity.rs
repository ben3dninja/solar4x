use bevy::{math::DVec3, prelude::*};

use crate::{
    core_plugin::{build_system, AppState, BodyInfo, EntityMapping, PrimaryBody},
    engine_plugin::{GameSpeed, Position, Velocity, SECONDS_PER_DAY},
};

/// Gravitationnal constant in km3kg-1d-2
const G: f64 = 6.6743e-11 * SECONDS_PER_DAY * SECONDS_PER_DAY * 1e-9;

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Loaded),
            setup_hill_spheres.after(build_system),
        )
        .add_systems(FixedUpdate, (apply_gravity_force, integrate_positions));
    }
}

#[derive(Component)]
pub struct Mass(pub f64);

#[derive(Component)]
pub struct GravityBound;

#[derive(Component)]
pub struct HillRadius(f64);

pub fn setup_hill_spheres(
    mut commands: Commands,
    mut query: Query<&BodyInfo>,
    primary: Query<(Entity, &BodyInfo), With<PrimaryBody>>,
    mapping: Res<EntityMapping>,
) {
    let mut queue = vec![(primary.single().1 .0.id, 0.)];
    let mut i = 0;
    while i < queue.len() {
        let (id, parent_mass) = queue[i];
        if let Some(entity) = mapping.id_mapping.get(&id) {
            if let Ok(BodyInfo(data)) = query.get_mut(*entity) {
                let radius = data.semimajor_axis
                    * (1. - data.eccentricity)
                    * (data.mass / (3. * (parent_mass + data.mass)))
                        .powf(1. / 3.)
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

#[derive(Component, Debug, Default)]
pub struct Acceleration(pub DVec3);

pub fn apply_gravity_force(
    mut gravity_bound: Query<(&Position, &mut Acceleration), With<GravityBound>>,
    massive_objects: Query<(&Position, &Mass, &HillRadius)>,
) {
    gravity_bound
        .par_iter_mut()
        .for_each(|(object_pos, mut acceleration)| {
            let mut acc = DVec3::ZERO;
            massive_objects
                .iter()
                .for_each(|(mass_pos, Mass(mass), HillRadius(radius))| {
                    let r = object_pos.0 - mass_pos.0;
                    let dist = r.length();
                    if dist < *radius {
                        acc -= r * G * *mass / (dist.powi(3));
                    }
                });
            acceleration.0 = acc;
        });
}

pub fn integrate_positions(
    mut query: Query<(&mut Position, &mut Velocity, &Acceleration), With<GravityBound>>,
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
) {
    // Gametime-days since last update
    let dt = time.delta_seconds_f64() * game_speed.0;
    query.par_iter_mut().for_each(|(mut pos, mut speed, acc)| {
        speed.0 += acc.0 * dt;
        pos.0 += speed.0 * dt;
    });
}
