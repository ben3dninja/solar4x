use bevy::prelude::*;
use bevy_ratatui::{event::KeyEvent, terminal::RatatuiContext, RatatuiPlugins};
use explorer_screen::ExplorerScreenPlugin;
use start_menu::{StartMenu, StartMenuContext, StartMenuEvent, StartMenuPlugin};

use crate::{
    client_plugin::ClientMode, core_plugin::LoadingState, keyboard::Keymap, spaceship::ShipID,
    utils::ecs::exit_on_error_if_app,
};

use self::{
    explorer_screen::{ExplorerContext, ExplorerEvent, ExplorerScreen},
    fleet_screen::{FleetContext, FleetScreen, FleetScreenEvent, FleetScreenPlugin},
};

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
        if !self.headless {
            app.add_plugins(RatatuiPlugins::default())
                .insert_resource(self.keymap.clone())
                .add_systems(
                    PostUpdate,
                    render.pipe(exit_on_error_if_app).in_set(UiUpdate),
                )
                .add_systems(PreUpdate, handle_input.before(change_screen));
        }
        app.add_plugins((StartMenuPlugin, ExplorerScreenPlugin, FleetScreenPlugin))
            .insert_resource(AppScreen::default())
            .configure_sets(PostUpdate, (ContextUpdate, UiUpdate).chain())
            .configure_sets(OnEnter(LoadingState::Loaded), UiInit)
            .add_event::<ChangeAppScreen>()
            .add_systems(PreUpdate, change_screen);
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextUpdate;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiUpdate;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiInit;

/// A resource storing the current screen and its associated context, with only one context valid at a time
/// In systems, checking the screen is done at the same time as acquiring the context so no run conditions are needed
#[allow(clippy::large_enum_variant)]
#[derive(Resource)]
pub enum AppScreen {
    StartMenu(StartMenuContext),
    Explorer(ExplorerContext),
    Fleet(FleetContext),
}
impl Default for AppScreen {
    fn default() -> Self {
        Self::StartMenu(StartMenuContext::default())
    }
}

#[derive(Event, Clone, Copy)]
pub enum ChangeAppScreen {
    StartMenu,
    Singleplayer,
    Multiplayer,
    Explorer,
    TrajectoryEditor(ShipID),
}

trait ScreenContext {
    type ScreenEvent: Event;
    type ScreenKeymap;

    fn read_input(
        &mut self,
        key_event: &KeyEvent,
        keymap: &Self::ScreenKeymap,
        internal_event: &mut Events<Self::ScreenEvent>,
    ) -> Option<ChangeAppScreen>;
}

fn handle_input(
    mut screen: ResMut<AppScreen>,
    mut next_screen: EventWriter<ChangeAppScreen>,
    mut key_reader: EventReader<KeyEvent>,
    mut explorer_events: ResMut<Events<ExplorerEvent>>,
    mut start_menu_events: ResMut<Events<StartMenuEvent>>,
    mut fleet_events: ResMut<Events<FleetScreenEvent>>,
    keymap: Res<Keymap>,
) {
    match screen.as_mut() {
        AppScreen::StartMenu(ctx) => key_reader.read().for_each(|e| {
            if let Some(s) = ctx.read_input(e, &keymap.start_menu, start_menu_events.as_mut()) {
                next_screen.send(s);
            }
        }),
        AppScreen::Explorer(ctx) => key_reader.read().for_each(|e| {
            if let Some(s) = ctx.read_input(e, &keymap.explorer, explorer_events.as_mut()) {
                next_screen.send(s);
            }
        }),
        AppScreen::Fleet(ctx) => key_reader.read().for_each(|e| {
            if let Some(s) = ctx.read_input(e, &keymap.fleet_screen, fleet_events.as_mut()) {
                next_screen.send(s);
            }
        }),
    }
}

fn change_screen(
    mut screen: ResMut<AppScreen>,
    mut next_screen: EventReader<ChangeAppScreen>,
    mut next_client_mode: ResMut<NextState<ClientMode>>,
) {
    for s in next_screen.read() {
        match s {
            ChangeAppScreen::StartMenu => {
                next_client_mode.set(ClientMode::None);
                *screen = AppScreen::StartMenu(StartMenuContext::default());
            }
            ChangeAppScreen::Explorer => {
                next_client_mode.set(ClientMode::Explorer);
            }
            ChangeAppScreen::Singleplayer => {
                next_client_mode.set(ClientMode::Singleplayer);
            }
            _ => todo!(),
        }
    }
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    mut screen: ResMut<AppScreen>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| match screen.as_mut() {
        AppScreen::StartMenu(ctx) => f.render_stateful_widget(StartMenu, f.size(), ctx),
        AppScreen::Explorer(ctx) => f.render_stateful_widget(ExplorerScreen, f.size(), ctx),
        AppScreen::Fleet(ctx) => f.render_stateful_widget(FleetScreen, f.size(), ctx),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

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
        app.update();
        app.update();
        let world = app.world_mut();
        assert!(matches!(
            *world.resource::<AppScreen>(),
            AppScreen::Explorer(_)
        ));
    }
}
