use bevy::{math::DVec3, prelude::*};

mod orbit;
mod influence;
mod leapfrog;
mod time;

/// Gravitationnal constant in km3kg-1d-2
pub const G: f64 = 6.6743e-11 * SECONDS_PER_DAY * SECONDS_PER_DAY * 1e-9;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Position(pub DVec3);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Velocity(pub DVec3);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((orbit::plugin, influence::plugin, leapfrog::plugin, time::plugin))
    }
}