use bevy::prelude::*;
use bevy_ratatui::{event::KeyEvent, RatatuiPlugins};

pub mod gui_plugin;
pub mod screen;
pub mod widget;

#[derive(Default)]
pub struct TuiPlugin {
    pub headless: bool,
    pub keymap: Keymap,
}

impl TuiPlugin {
    pub fn testing() -> Self {
        Self {
            headless: true,
            ..default()
        }
    }
}

impl Plugin for TuiPlugin {
    fn build(&self, app: &mut App) {
        if self.headless {
            app.add_event::<KeyEvent>();
        } else {
            app.add_plugins(RatatuiPlugins::default()).add_systems(
                PostUpdate,
                render.pipe(exit_on_error_if_app).in_set(UiUpdate),
            );
        }
        app.insert_resource(self.keymap.clone())
            .configure_sets(PostUpdate, (ContextUpdate, UiUpdate).chain())
            .configure_sets(OnEnter(LoadingState::Loaded), UiInit)
            .add_systems(
                PreUpdate,
                update_previous_screen.run_if(resource_changed::<NextState<AppScreen>>),
            )
            .add_systems(
                Update,
                clear_key_events
                    .before(InputReading)
                    .run_if(state_changed::<AppScreen>),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextUpdate;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiUpdate;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiInit;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CreateScreen;

fn clear_key_events(mut events: ResMut<Events<KeyEvent>>) {
    events.clear();
}

#[cfg(test)]
mod tests {
    use bevy::{app::App, state::state::State};

    use crate::{
        client::{ClientMode, ClientPlugin},
        ui::{AppScreen, TuiPlugin},
    };

    #[test]
    fn test_change_screen() {
        let mut app = App::new();
        app.add_plugins((
            ClientPlugin::testing().in_mode(ClientMode::Explorer),
            TuiPlugin::testing(),
        ));
        // One update to enter the explorer mode
        app.update();
        // One update to create the body system
        app.update();
        // One update to enter the screen
        app.update();
        let world = app.world_mut();
        assert!(matches!(
            *world.resource::<State<AppScreen>>().get(),
            AppScreen::Explorer
        ));
    }
}
