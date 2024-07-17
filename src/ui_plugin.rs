use bevy::prelude::*;
use bevy_ratatui::{event::KeyEvent, terminal::RatatuiContext, RatatuiPlugins};
use explorer_screen::ExplorerScreenPlugin;
use start_menu::{StartMenu, StartMenuContext, StartMenuPlugin};

use crate::{
    core_plugin::{InputReading, LoadingState},
    keyboard::Keymap,
    spaceship::ShipID,
    utils::ecs::exit_on_error_if_app,
};

use self::{
    editor_screen::{EditorContext, EditorPlugin, EditorScreen},
    explorer_screen::{ExplorerContext, ExplorerScreen},
    fleet_screen::{FleetContext, FleetScreen, FleetScreenPlugin},
    space_map_plugin::SpaceMap,
};

pub mod editor_gui;
pub mod editor_screen;
pub mod explorer_screen;
pub mod fleet_screen;
pub mod gui_plugin;
pub mod info_widget;
pub mod search_plugin;
pub mod space_map_plugin;
pub mod start_menu;
pub mod tree_widget;

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
        app.add_plugins((
            StartMenuPlugin,
            ExplorerScreenPlugin,
            FleetScreenPlugin,
            EditorPlugin,
        ))
        .insert_resource(self.keymap.clone())
        .init_state::<AppScreen>()
        .init_resource::<PreviousScreen>()
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

/// A resource storing the current screen
/// Set this to change screen, the appropriate context is automatically generated when the app is ready
/// (for example when the bodies have been imported)
#[derive(States, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum AppScreen {
    #[default]
    StartMenu,
    Explorer,
    Fleet,
    Editor(ShipID),
}

#[derive(Resource, Default, Debug)]
pub struct PreviousScreen(pub AppScreen);

fn clear_key_events(mut events: ResMut<Events<KeyEvent>>) {
    events.clear();
}

fn update_previous_screen(
    next: Res<NextState<AppScreen>>,
    current: Res<State<AppScreen>>,
    mut previous: ResMut<PreviousScreen>,
) {
    if let NextState::Pending(_) = next.as_ref() {
        previous.0 = *current.get();
    }
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    screen: Res<State<AppScreen>>,
    start_menu: Option<ResMut<StartMenuContext>>,
    explorer: Option<ResMut<ExplorerContext>>,
    fleet: Option<ResMut<FleetContext>>,
    editor: Option<ResMut<EditorContext>>,
    space_map: Option<ResMut<SpaceMap>>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| match screen.get() {
        AppScreen::StartMenu => {
            f.render_stateful_widget(StartMenu, f.size(), start_menu.unwrap().as_mut())
        }
        AppScreen::Explorer => {
            if let Some(mut explorer) = explorer {
                f.render_stateful_widget(
                    ExplorerScreen {
                        map: space_map.unwrap().as_mut(),
                    },
                    f.size(),
                    explorer.as_mut(),
                )
            }
        }
        AppScreen::Fleet => {
            f.render_stateful_widget(FleetScreen, f.size(), fleet.unwrap().as_mut())
        }
        AppScreen::Editor(_) => {
            f.render_stateful_widget(EditorScreen, f.size(), editor.unwrap().as_mut())
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::{app::App, state::state::State};

    use crate::{
        client_plugin::{ClientMode, ClientPlugin},
        ui_plugin::{AppScreen, TuiPlugin},
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
