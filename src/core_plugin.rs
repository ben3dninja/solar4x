use bevy::{app::AppExit, prelude::*, state::app::StatesPlugin, time::TimePlugin};

/// This plugin's role is to handle body system creation
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CoreEvent>()
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
