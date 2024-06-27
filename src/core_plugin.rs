use bevy::{app::AppExit, prelude::*, time::TimePlugin, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::{
    bodies::{
        body_data::{BodyData, BodyType},
        body_id::BodyID,
    },
    engine_plugin::{update_global, EllipticalOrbit, Position},
    tui_plugin::InitializeUiSet,
    utils::de::read_main_bodies,
};
pub struct CorePlugin;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub enum BodiesConfig {
    SmallestBodyType(BodyType),
    IDs(Vec<BodyID>),
}

impl Default for BodiesConfig {
    fn default() -> Self {
        BodiesConfig::SmallestBodyType(BodyType::Planet)
    }
}

impl BodiesConfig {
    fn into_filter(self) -> Box<dyn FnMut(&BodyData) -> bool> {
        match self {
            BodiesConfig::SmallestBodyType(body_type) => {
                Box::new(move |data: &BodyData| data.body_type <= body_type)
            }
            BodiesConfig::IDs(v) => Box::new(move |data: &BodyData| v.contains(&data.id)),
        }
    }
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TaskPoolPlugin::default(),
            TypeRegistrationPlugin,
            FrameCountPlugin,
            TimePlugin,
        ))
        .insert_state(AppState::Setup)
        .add_event::<CoreEvent>()
        .configure_sets(Update, GameSet.run_if(in_state(AppState::Game)))
        .configure_sets(PreUpdate, GameSet.run_if(in_state(AppState::Game)))
        .configure_sets(PostUpdate, GameSet.run_if(in_state(AppState::Game)))
        .configure_sets(FixedUpdate, GameSet.run_if(in_state(AppState::Game)))
        .configure_sets(
            OnEnter(AppState::Game),
            InitializeUiSet.after(update_global),
        )
        .add_systems(OnEnter(AppState::Game), (build_system).chain())
        .add_systems(Update, handle_core_events.in_set(GameSet));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSet;

#[derive(States, Debug, PartialEq, Eq, Clone, Hash)]
pub enum AppState {
    Setup,
    Game,
}
#[derive(Resource)]
pub struct PrimaryBody(pub BodyID);

#[derive(Resource)]
pub struct EntityMapping {
    pub id_mapping: HashMap<BodyID, Entity>,
}

#[derive(Component, Debug)]
pub struct BodyInfo(pub BodyData);

pub fn start_game(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::Game);
}

pub fn build_system(mut commands: Commands, config: Res<BodiesConfig>) {
    let bodies: Vec<_> = read_main_bodies()
        .expect("Failed to read bodies")
        .into_iter()
        .filter(config.clone().into_filter())
        .collect();
    let primary_body = bodies
        .iter()
        .find(|data| data.host_body.is_none())
        .expect("no primary body found")
        .id;
    commands.insert_resource(PrimaryBody(primary_body));
    let mut id_mapping = HashMap::new();
    for data in bodies {
        let id = data.id;
        let entity = commands.spawn((
            Position::default(),
            EllipticalOrbit::from(&data),
            BodyInfo(data),
        ));
        id_mapping.insert(id, entity.id());
    }
    commands.insert_resource(EntityMapping { id_mapping });
}

#[derive(Event)]
pub enum CoreEvent {
    Quit,
}

fn handle_core_events(mut reader: EventReader<CoreEvent>, mut quit_writer: EventWriter<AppExit>) {
    for event in reader.read() {
        match event {
            CoreEvent::Quit => {
                quit_writer.send_default();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        bodies::body_data::BodyType,
        core_plugin::{BodiesConfig, EntityMapping},
        engine_plugin::EllipticalOrbit,
        standalone_plugin::StandalonePlugin,
    };

    use super::BodyInfo;

    #[test]
    fn test_build_system() {
        let mut app = App::new();
        app.add_plugins(StandalonePlugin(BodiesConfig::SmallestBodyType(
            BodyType::Planet,
        )));
        app.update();
        let mut world = app.world;
        assert_eq!(world.resource::<EntityMapping>().id_mapping.len(), 9);
        assert_eq!(world.query::<&BodyInfo>().iter(&world).len(), 9);
        let (orbit, BodyInfo(data)) = world
            .query::<(&EllipticalOrbit, &BodyInfo)>()
            .iter(&world)
            .find(|(_, BodyInfo(data))| data.id == "terre".into())
            .unwrap();
        assert_eq!(orbit.semimajor_axis, 149598023.);
        assert_eq!(data.semimajor_axis, 149598023);
    }
}
