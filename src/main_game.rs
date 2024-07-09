use bevy::{math::DVec3, prelude::*};
use trajectory::update_speed;

use crate::{
    engine_plugin::{Position, ToggleTime, Velocity},
    gravity::{integrate_positions, Acceleration, GravityBound},
    spaceship::{ShipID, ShipInfo, ShipsMapping},
};

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameStage {
    Preparation,
    Action,
}

pub mod trajectory;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShipsMapping::default())
            .insert_state(GameStage::Preparation)
            .add_event::<ShipEvent>()
            .add_systems(OnEnter(GameStage::Action), enable_time)
            .add_systems(OnEnter(GameStage::Preparation), disable_time)
            .add_systems(
                FixedUpdate,
                update_speed
                    .run_if(in_state(GameStage::Action))
                    .before(integrate_positions),
            )
            .add_systems(Update, handle_ship_events);
    }
}

pub fn enable_time(mut toggle: ResMut<ToggleTime>) {
    toggle.0 = true;
}
pub fn disable_time(mut toggle: ResMut<ToggleTime>) {
    toggle.0 = false;
}

#[derive(Event)]
pub enum ShipEvent {
    Create(ShipInfo),
    Remove(ShipID),
}

pub fn handle_ship_events(
    mut commands: Commands,
    mut reader: EventReader<ShipEvent>,
    mut ships: ResMut<ShipsMapping>,
) {
    for event in reader.read() {
        match event {
            ShipEvent::Create(info) => {
                ships.0.entry(info.id).or_insert(
                    commands
                        .spawn((
                            info.clone(),
                            Position(info.spawn_pos),
                            Velocity(info.spawn_speed),
                            Acceleration(DVec3::ZERO),
                            GravityBound,
                        ))
                        .id(),
                );
            }
            ShipEvent::Remove(id) => {
                ships.0.remove(id).map(|e| commands.entity(e).despawn());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{app::App, math::DVec3};

    use crate::{
        client_plugin::{ClientMode, ClientPlugin},
        core_plugin::BodiesConfig,
        engine_plugin::EnginePlugin,
        main_game::ShipEvent,
        spaceship::{ShipID, ShipInfo, ShipsMapping},
    };

    use super::GamePlugin;

    #[test]
    fn test_handle_ship_events() {
        let mut app = App::new();
        app.add_plugins((
            ClientPlugin::testing(BodiesConfig::default(), ClientMode::Singleplayer),
            EnginePlugin,
            GamePlugin,
        ));
        app.update();
        app.world.send_event(ShipEvent::Create(ShipInfo {
            id: ShipID::from("s").unwrap(),
            spawn_pos: DVec3::new(1e6, 0., 0.),
            spawn_speed: DVec3::new(0., 1e6, 0.),
        }));
        app.update();
        let world = &mut app.world;
        assert_eq!(world.resource::<ShipsMapping>().0.len(), 1);
        assert_eq!(world.query::<&ShipInfo>().iter(world).len(), 1);
    }
}
