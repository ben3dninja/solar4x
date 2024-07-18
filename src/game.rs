use std::time::Duration;

use bevy::prelude::*;
use trajectory::handle_thrusts;

use crate::{
    client::ClientMode,
    core_plugin::{BodiesMapping, BodyInfo, EventHandling, LoadingState, PrimaryBody},
    orbit::{Position, ToggleTime, Velocity},
    influence::{compute_influence, HillRadius, InfluencePlugin},
    leapfrog::{get_acceleration, Acceleration, LeapfrogPlugin, LeapfrogSystems},
    spaceship::{ShipID, ShipInfo, ShipsMapping},
    utils::de::TempDirectory,
    TPS,
};

use self::trajectory::{TrajectoriesDirectory, TrajectoryEvent, VelocityUpdate, TRAJECTORIES_PATH};

pub mod trajectory;

/// This plugin's role is to handle everything that is about the main game, and that is common to both the server and the client
#[derive(Default)]
pub struct GamePlugin {
    pub testing: bool,
}

impl GamePlugin {
    pub fn testing() -> Self {
        Self { testing: true }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let path = if self.testing {
            let dir = TempDirectory::default();
            let path = dir.0.path().to_owned();
            app.insert_resource(dir);
            path
        } else {
            TRAJECTORIES_PATH.into()
        };
        app.add_plugins(InfluencePlugin)
            .add_plugins(LeapfrogPlugin)
            .add_computed_state::<InGame>()
            .add_sub_state::<GameStage>()
            .insert_resource(TrajectoriesDirectory(path))
            .add_event::<TrajectoryEvent>()
            .add_event::<VelocityUpdate>()
            .add_systems(OnEnter(GameStage::Action), enable_time)
            .add_systems(OnEnter(GameStage::Preparation), disable_time)
            .add_systems(
                FixedUpdate,
                handle_thrusts
                    .run_if(in_state(GameStage::Action))
                    .before(LeapfrogSystems),
            );
    }
}

/// This state represents whether the app is running the main game (singleplayer or multiplayer) or not, and is loaded
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InGame;

impl ComputedStates for InGame {
    type SourceStates = (Option<ClientMode>, LoadingState);

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        if !matches!(sources.1, LoadingState::Loaded) {
            None
        } else {
            match sources.0 {
                Some(ClientMode::Singleplayer | ClientMode::Multiplayer) | None => Some(InGame),
                _ => None,
            }
        }
    }
}

#[derive(SubStates, Debug, Hash, PartialEq, Eq, Clone, Default)]
#[source(InGame = InGame)]
pub enum GameStage {
    #[default]
    Preparation,
    Action,
}

impl std::fmt::Display for GameStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameStage::Preparation => "Preparation",
                GameStage::Action => "Action",
            }
        )
    }
}



pub fn enable_time(mut toggle: ResMut<ToggleTime>) {
    toggle.0 = true;
}
pub fn disable_time(mut toggle: ResMut<ToggleTime>) {
    toggle.0 = false;
}



#[cfg(test)]
mod tests {
    use bevy::{app::App, math::DVec3, state::state::State};

    use crate::{
        client::{ClientMode, ClientPlugin},
        game::{GameStage, InGame, ShipEvent},
        spaceship::{ShipID, ShipInfo, ShipsMapping},
    };

    fn new_app() -> App {
        let mut app = App::new();
        app.add_plugins(ClientPlugin::testing().in_mode(ClientMode::Singleplayer));
        app.update();
        app.update();
        app
    }

    #[test]
    fn test_handle_ship_events() {
        let mut app = new_app();

        app.world_mut().send_event(ShipEvent::Create(ShipInfo {
            id: ShipID::from("s").unwrap(),
            spawn_pos: DVec3::new(1e6, 0., 0.),
            spawn_speed: DVec3::new(0., 1e6, 0.),
        }));
        app.update();
        let world = app.world_mut();
        assert_eq!(world.resource::<ShipsMapping>().0.len(), 1);
        assert_eq!(world.query::<&ShipInfo>().iter(world).len(), 1);
    }

    #[test]
    fn test_states() {
        let app = new_app();
        assert!(app.world().contains_resource::<State<InGame>>());
        assert_eq!(
            *app.world().resource::<State<GameStage>>(),
            GameStage::Preparation
        );
    }
}
