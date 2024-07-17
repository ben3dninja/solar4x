use bevy::{app::AppExit, prelude::*, state::app::StatesPlugin, time::TimePlugin, utils::HashMap};

use crate::{
    bodies::{bodies_config::BodiesConfig, body_data::BodyData, body_id::BodyID},
    engine_plugin::{update_global, EllipticalOrbit, Position, ToggleTime, Velocity},
    influence::Mass,
    utils::de::read_main_bodies,
    STPS,
};
/// This plugin's role is to handle body system creation
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // required plugins for the app to work. If there is no gui, we still have to add a schedulerunner plugin (see bevy default and minimal plugin sets)
            TaskPoolPlugin::default(),
            TypeRegistrationPlugin,
            FrameCountPlugin,
            TimePlugin,
            StatesPlugin,
        ))
        .init_state::<LoadingState>()
        .insert_resource(Time::<Fixed>::from_hz(STPS))
        .add_event::<CoreEvent>()
        .configure_sets(Update, LoadedSet.run_if(in_state(LoadingState::Loaded)))
        .configure_sets(PreUpdate, LoadedSet.run_if(in_state(LoadingState::Loaded)))
        .configure_sets(PostUpdate, LoadedSet.run_if(in_state(LoadingState::Loaded)))
        .configure_sets(
            FixedUpdate,
            LoadedSet.run_if(in_state(LoadingState::Loaded)),
        )
        .configure_sets(Update, (InputReading, EventHandling).chain())
        .add_systems(
            OnEnter(LoadingState::Loading),
            (build_system, insert_system_size.after(update_global)),
        )
        .add_systems(OnEnter(LoadingState::Unloading), clear_system)
        .add_systems(Update, handle_core_events.in_set(EventHandling));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadedSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputReading;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventHandling;

/// This state represents whether or not a planetary system is loaded in game.
/// For server, is is automatically the case, but for a client a system is loaded only if one is connected to a server,
/// or if the singleplayer or explore modes have been launched
#[derive(States, Debug, PartialEq, Eq, Clone, Hash, Default)]
pub enum LoadingState {
    #[default]
    NotLoaded,
    Unloading,
    Loading,
    Loaded,
}

#[derive(Component)]
pub struct PrimaryBody;

#[derive(Resource)]
pub struct BodiesMapping(pub HashMap<BodyID, Entity>);

#[derive(Component, Debug, Clone)]
pub struct BodyInfo(pub BodyData);

#[derive(Resource)]
pub struct SystemSize(pub f64);

pub fn build_system(
    mut commands: Commands,
    config: Res<BodiesConfig>,
    mut loading_state: ResMut<NextState<LoadingState>>,
) {
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
    loading_state.set(LoadingState::Loaded);
}

pub fn insert_system_size(mut commands: Commands, body_positions: Query<&Position>) {
    let system_size = body_positions
        .iter()
        .map(|pos| pos.0.length())
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();
    commands.insert_resource(SystemSize(system_size));
}

fn clear_system(
    mut commands: Commands,
    mapping: Res<BodiesMapping>,
    mut toggle_time: Option<ResMut<ToggleTime>>,
    mut loading_state: ResMut<NextState<LoadingState>>,
) {
    for entity in mapping.0.values() {
        commands.entity(*entity).despawn();
    }
    commands.remove_resource::<BodiesMapping>();
    if let Some(toggle) = toggle_time.as_mut() {
        toggle.0 = false;
    }
    loading_state.set(LoadingState::NotLoaded);
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
    use bevy::{app::App, ecs::query::With};

    use crate::{
        bodies::body_id::id_from,
        client_plugin::{ClientMode, ClientPlugin},
        core_plugin::{BodiesMapping, PrimaryBody},
        engine_plugin::EllipticalOrbit,
    };

    use super::BodyInfo;

    #[test]
    fn test_build_system() {
        let mut app = App::new();
        app.add_plugins(ClientPlugin::testing().in_mode(ClientMode::Explorer));
        app.update();
        app.update();

        let world = app.world_mut();
        assert_eq!(world.resource::<BodiesMapping>().0.len(), 9);
        assert_eq!(world.query::<&BodyInfo>().iter(world).len(), 9);
        let (orbit, BodyInfo(data)) = world
            .query::<(&EllipticalOrbit, &BodyInfo)>()
            .iter(world)
            .find(|(_, BodyInfo(data))| data.id == id_from("terre"))
            .unwrap();
        assert_eq!(orbit.semimajor_axis, 149598023.);
        assert_eq!(data.semimajor_axis, 149598023.);
        assert_eq!(
            world
                .query_filtered::<&BodyInfo, With<PrimaryBody>>()
                .single(world)
                .0
                .id,
            id_from("soleil")
        )
    }
}
