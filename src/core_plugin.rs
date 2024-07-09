use bevy::{app::AppExit, prelude::*, time::TimePlugin, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::{
    bodies::{
        body_data::{BodyData, BodyType},
        body_id::BodyID,
    },
    engine_plugin::{EllipticalOrbit, Position, ToggleTime, Velocity},
    gravity::Mass,
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
            // required plugins for the app to work. If there is no gui, we still have to add a schedulerunner plugin (see bevy default and minimal plugin sets)
            TaskPoolPlugin::default(),
            TypeRegistrationPlugin,
            FrameCountPlugin,
            TimePlugin,
        ))
        .insert_state(AppState::Setup)
        .add_event::<CoreEvent>()
        .configure_sets(Update, SimulationSet.run_if(in_state(AppState::Loaded)))
        .configure_sets(PreUpdate, SimulationSet.run_if(in_state(AppState::Loaded)))
        .configure_sets(PostUpdate, SimulationSet.run_if(in_state(AppState::Loaded)))
        .configure_sets(
            FixedUpdate,
            SimulationSet.run_if(in_state(AppState::Loaded)),
        )
        .configure_sets(
            OnEnter(AppState::Loaded),
            (SystemInitSet, UiInitSet).chain(),
        )
        .add_systems(
            OnEnter(AppState::Loaded),
            build_system.in_set(SystemInitSet),
        )
        .add_systems(OnExit(AppState::Loaded), clear_system)
        .add_systems(Update, handle_core_events);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemInitSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiInitSet;

/// This state represents whether or not a planetary system is loaded in game.
/// For server, is is automatically the case, but for a client a system is loaded only if one is connected to a server,
/// or if the singleplayer or explore modes have been launched
#[derive(States, Debug, PartialEq, Eq, Clone, Hash)]
pub enum AppState {
    Setup,
    Loaded,
}

#[derive(Component)]
pub struct PrimaryBody;

#[derive(Resource)]
pub struct BodiesMapping(pub HashMap<BodyID, Entity>);

#[derive(Component, Debug, Clone)]
pub struct BodyInfo(pub BodyData);

pub fn start_game(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::Loaded);
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
    let mut id_mapping = HashMap::new();
    for data in bodies {
        let id = data.id;
        let mut entity = commands.spawn((
            Position::default(),
            EllipticalOrbit::from(&data),
            Mass(data.mass),
            BodyInfo(data),
            Velocity::default(),
        ));
        if id == primary_body {
            entity.insert(PrimaryBody);
        }
        id_mapping.insert(id, entity.id());
    }
    commands.insert_resource(BodiesMapping(id_mapping));
}

fn clear_system(
    mut commands: Commands,
    mapping: Res<BodiesMapping>,
    mut toggle_time: Option<ResMut<ToggleTime>>,
) {
    for entity in mapping.0.values() {
        commands.entity(*entity).despawn();
    }
    commands.remove_resource::<BodiesMapping>();
    if let Some(toggle) = toggle_time.as_mut() {
        toggle.0 = false;
    }
}

#[derive(Event, Clone, Copy)]
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
    use bevy::{app::App, ecs::query::With, prelude::NextState};

    use crate::{
        bodies::body_data::BodyType,
        client_plugin::{ClientMode, ClientPlugin},
        core_plugin::{BodiesConfig, BodiesMapping, PrimaryBody},
        engine_plugin::EllipticalOrbit,
    };

    use super::BodyInfo;

    #[test]
    fn test_build_system() {
        let mut app = App::new();
        app.add_plugins(ClientPlugin::testing(BodiesConfig::SmallestBodyType(
            BodyType::Planet,
        )));
        app.update();
        app.update();
        app.world
            .resource_mut::<NextState<ClientMode>>()
            .set(ClientMode::Explorer);
        app.update();
        let mut world = app.world;
        assert_eq!(world.resource::<BodiesMapping>().0.len(), 9);
        assert_eq!(world.query::<&BodyInfo>().iter(&world).len(), 9);
        let (orbit, BodyInfo(data)) = world
            .query::<(&EllipticalOrbit, &BodyInfo)>()
            .iter(&world)
            .find(|(_, BodyInfo(data))| data.id == "terre".into())
            .unwrap();
        assert_eq!(orbit.semimajor_axis, 149598023.);
        assert_eq!(data.semimajor_axis, 149598023.);
        assert_eq!(
            world
                .query_filtered::<&BodyInfo, With<PrimaryBody>>()
                .single(&world)
                .0
                .id,
            "soleil".into()
        )
    }
}
