use bevy::{app::AppExit, prelude::*, state::app::StatesPlugin, time::TimePlugin, utils::HashMap};

use crate::{
    bodies::{bodies_config::BodiesConfig, body_data::BodyData, body_id::BodyID},
    orbit::{update_global, EllipticalOrbit, Position, ToggleTime, Velocity},
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


#[derive(Resource)]
pub struct SystemSize(pub f64);

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

