use bevy::prelude::*;
use bevy_ratatui::{
    error::exit_on_error, event::KeyEvent, terminal::RatatuiContext, RatatuiPlugins,
};
use explorer_screen::ExplorerScreenPlugin;
use start_menu::{StartMenu, StartMenuContext, StartMenuEvent, StartMenuPlugin};

use crate::{client_plugin::ClientMode, keyboard::Keymap};

use self::explorer_screen::{ExplorerContext, ExplorerEvent, ExplorerScreen};

pub mod explorer_screen;
pub mod info_widget;
pub mod search_plugin;
pub mod space_map_plugin;
pub mod start_menu;
pub mod tree_widget;

#[derive(Default)]
pub struct TuiPlugin {
    pub headless: bool,
    pub start_in_explorer: bool,
    pub keymap: Keymap,
}

impl TuiPlugin {
    pub fn testing() -> TuiPlugin {
        TuiPlugin {
            headless: true,
            start_in_explorer: true,
            ..default()
        }
    }
}

impl Plugin for TuiPlugin {
    fn build(&self, app: &mut App) {
        if !self.headless {
            app.add_plugins(RatatuiPlugins::default())
                .insert_resource(self.keymap.clone())
                .add_systems(PostUpdate, render.pipe(exit_on_error))
                .add_systems(PreUpdate, handle_input.before(change_screen));
        }
        app.add_plugins((StartMenuPlugin, ExplorerScreenPlugin))
            .insert_resource(AppScreen::default())
            .add_event::<ChangeAppScreen>()
            .add_systems(PreUpdate, change_screen);
        if self.start_in_explorer {
            // Since we only send the event and don't do the change manually, we have to wait for 2 schedule updates to get the new screen.
            // Hence the initial double update call in the tests
            app.world.send_event(ChangeAppScreen::Explorer);
        }
    }
}

/// A resource storing the current screen and its associated context, with only one context valid at a time
/// In systems, checking the screen is done at the same time as acquiring the context so no run conditions are needed
#[allow(clippy::large_enum_variant)]
#[derive(Resource)]
pub enum AppScreen {
    StartMenu(StartMenuContext),
    Explorer(ExplorerContext),
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
    mut main_screen_events: ResMut<Events<StartMenuEvent>>,
    keymap: Res<Keymap>,
) {
    match screen.as_mut() {
        AppScreen::StartMenu(ctx) => key_reader.read().for_each(|e| {
            if let Some(s) = ctx.read_input(e, &keymap.start_menu, main_screen_events.as_mut()) {
                next_screen.send(s);
            }
        }),
        AppScreen::Explorer(ctx) => key_reader.read().for_each(|e| {
            if let Some(s) = ctx.read_input(e, &keymap.explorer, explorer_events.as_mut()) {
                next_screen.send(s);
            }
        }),
    }
}

fn change_screen<'a>(
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
            _ => todo!(),
        }
    }
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    mut screen: ResMut<AppScreen>,
    explorer_screen: Res<ExplorerScreen>,
    start_menu: Res<StartMenu>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| match screen.as_mut() {
        AppScreen::StartMenu(ctx) => f.render_stateful_widget(start_menu.as_ref(), f.size(), ctx),
        AppScreen::Explorer(ctx) => {
            f.render_stateful_widget(explorer_screen.as_ref(), f.size(), ctx)
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        client_plugin::ClientPlugin,
        tui_plugin::{AppScreen, TuiPlugin},
    };

    #[test]
    fn test_change_screen() {
        let mut app = App::new();
        app.add_plugins((ClientPlugin::default(), TuiPlugin::testing()));
        app.update();
        app.update();
        let world = &mut app.world;
        assert!(matches!(
            *world.resource::<AppScreen>(),
            AppScreen::Explorer(_)
        ));
    }
}
