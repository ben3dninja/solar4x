use bevy::prelude::*;

use crate::{
    engine_plugin::ToggleTime,
    gravity::integrate_positions,
    spaceship::{ShipEvent, ShipsMapping},
    trajectory::update_speed,
};

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameStage {
    Preparation,
    Action,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShipsMapping::default())
            .insert_state(GameStage::Preparation)
            .add_systems(OnEnter(GameStage::Action), enable_time)
            .add_systems(OnEnter(GameStage::Preparation), disable_time)
            .add_systems(
                FixedUpdate,
                update_speed
                    .run_if(in_state(GameStage::Action))
                    .before(integrate_positions),
            );
    }
}

pub fn enable_time(mut toggle: ResMut<ToggleTime>) {
    toggle.0 = true;
}
pub fn disable_time(mut toggle: ResMut<ToggleTime>) {
    toggle.0 = false;
}

pub fn handle_ship_events(
    mut commands: Commands,
    mut reader: EventReader<ShipEvent>,
    mut ships: ResMut<ShipsMapping>,
) {
    for event in reader.read() {
        match event {
            ShipEvent::Create(info) => {
                ships
                    .0
                    .entry(info.id)
                    .or_insert(commands.spawn(info.clone()).id());
            }
            ShipEvent::Remove(id) => {
                ships.0.remove(id).map(|e| commands.entity(e).despawn());
            }
        }
    }
}
