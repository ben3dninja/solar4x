use bevy::{prelude::*, state::app::StatesPlugin, time::TimePlugin};

use crate::{
    client::ClientMode,
    objects::{prelude::BodiesMapping, ships::ShipsMapping, ObjectsUpdate},
    physics::{
        influence::InfluenceUpdate, orbit::OrbitsUpdate, prelude::ToggleTime, PhysicsPlugin,
    },
    ui::gui_plugin::GUIUpdate,
};

pub mod prelude {
    pub use super::{GameStage, InGame, LoadingState};
}

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
        app.add_plugins((
            // required plugins for the app to work. If there is no gui, we still have to add a schedulerunner plugin (see bevy default and minimal plugin sets)
            TaskPoolPlugin::default(),
            TypeRegistrationPlugin,
            FrameCountPlugin,
            TimePlugin,
            StatesPlugin,
        ))
        .add_plugins(PhysicsPlugin)
        .add_computed_state::<InGame>()
        .add_sub_state::<GameStage>()
        .init_state::<LoadingState>()
        .configure_sets(
            OnEnter(LoadingState::Loaded),
            (ObjectsUpdate, OrbitsUpdate, InfluenceUpdate, GUIUpdate),
        )
        .add_systems(OnExit(LoadingState::Loaded), clear_loaded)
        .add_systems(OnEnter(GameStage::Action), enable_time)
        .add_systems(OnEnter(GameStage::Preparation), disable_time);
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

/// This state represents whether or not bodies and ships are loaded in game.
/// For server, is is automatically the case, but for a client a system is loaded only if one is connected to a server,
/// or if the singleplayer or explore modes have been launched
#[derive(States, Debug, PartialEq, Eq, Clone, Hash, Default)]
pub enum LoadingState {
    #[default]
    NotLoaded,
    Loaded,
}

#[derive(Component)]
pub struct ClearOnUnload;

fn clear_loaded(mut commands: Commands, query: Query<Entity, With<ClearOnUnload>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<BodiesMapping>();
    commands.remove_resource::<ShipsMapping>();
}

fn enable_time(mut toggle: ResMut<ToggleTime>) {
    toggle.0 = true;
}
fn disable_time(mut toggle: ResMut<ToggleTime>) {
    toggle.0 = false;
}

#[cfg(test)]
mod tests {
    use bevy::{app::App, math::DVec3, state::state::State};

    use crate::{objects::ships::ShipEvent, prelude::*};

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
