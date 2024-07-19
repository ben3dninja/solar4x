use bevy::prelude::*;
use bevy_ratatui::{event::KeyEvent, RatatuiPlugins};

use crate::input::prelude::Keymap;

pub mod gui;
pub mod screen;
pub mod widget;

pub mod prelude {
    pub use super::{screen::AppScreen, EventHandling, InputReading, TuiPlugin};
}

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
            app.add_plugins(RatatuiPlugins::default());
        }
        app.insert_resource(self.keymap.clone())
            .configure_sets(PostUpdate, (UiUpdate, Render).chain())
            .configure_sets(Update, (InputReading, EventHandling).chain());
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputReading;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventHandling;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiUpdate;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Render;

#[cfg(test)]
mod tests {
    use bevy::{app::App, state::state::State};

    use crate::prelude::*;

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
