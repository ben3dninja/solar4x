use bevy::prelude::*;
use bevy_ratatui::{
    error::exit_on_error, event::KeyEvent, terminal::RatatuiContext, RatatuiPlugins,
};
use explorer_screen::ExplorerPlugin;
use main_screen::{MainScreen, MainScreenContext, MainScreenEvent, MainScreenPlugin};

use crate::{
    core_plugin::{BodyInfo, GameSet, PrimaryBody},
    engine_plugin::Position,
    keyboard::{ExplorerKeymap, MainScreenKeymap},
};

use self::explorer_screen::{ExplorerContext, ExplorerEvent, ExplorerScreen};

pub mod explorer_screen;
pub mod info_plugin;
pub mod main_screen;
pub mod search_plugin;
pub mod space_map_plugin;
pub mod tree_plugin;

pub struct TuiPlugin {
    headless: bool,
    start_in_explorer: bool,
}

impl Default for TuiPlugin {
    fn default() -> Self {
        Self {
            headless: false,
            start_in_explorer: false,
        }
    }
}

impl TuiPlugin {
    pub fn testing() -> TuiPlugin {
        TuiPlugin {
            headless: true,
            start_in_explorer: true,
        }
    }
}

impl Plugin for TuiPlugin {
    fn build(&self, app: &mut App) {
        if !self.headless {
            app.add_plugins(RatatuiPlugins::default())
                .insert_resource(Keymap::default())
                .add_systems(PostUpdate, render.pipe(exit_on_error).in_set(GameSet))
                .add_systems(
                    PreUpdate,
                    handle_input.before(change_screen).in_set(GameSet),
                );
        }
        app.add_plugins((MainScreenPlugin, ExplorerPlugin))
            .insert_resource(AppScreen::default())
            .add_event::<ChangeAppScreen>()
            .add_systems(PreUpdate, change_screen.in_set(GameSet));
        if self.start_in_explorer {
            app.world.send_event(ChangeAppScreen::Explorer);
        }
    }
}

#[derive(SystemSet, Debug, Clone, Hash, PartialEq, Eq)]
pub struct UiInitSet;

#[derive(Resource)]
pub enum AppScreen {
    StartMenu(MainScreenContext),
    Explorer(ExplorerContext),
}
impl Default for AppScreen {
    fn default() -> Self {
        Self::StartMenu(MainScreenContext::default())
    }
}

#[derive(Event, Clone, Copy)]
pub enum ChangeAppScreen {
    StartMenu,
    Singleplayer,
    Multiplayer,
    Explorer,
}

#[derive(Resource, Default)]
pub struct Keymap {
    explorer: ExplorerKeymap,
    main_screen: MainScreenKeymap,
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
    mut main_screen_events: ResMut<Events<MainScreenEvent>>,
    keymap: Res<Keymap>,
) {
    match screen.as_mut() {
        AppScreen::StartMenu(ctx) => key_reader.read().for_each(|e| {
            if let Some(s) = ctx.read_input(e, &keymap.main_screen, main_screen_events.as_mut()) {
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
    primary: Query<Entity, With<PrimaryBody>>,
    bodies: Query<(&'a BodyInfo, &'a Position)>,
) {
    for s in next_screen.read() {
        *screen = match s {
            ChangeAppScreen::StartMenu => AppScreen::StartMenu(MainScreenContext::default()),
            ChangeAppScreen::Explorer => {
                AppScreen::Explorer(ExplorerContext::new(primary.single(), &bodies))
            }
            _ => todo!(),
        }
    }
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    mut screen: ResMut<AppScreen>,
    explorer_screen: Res<ExplorerScreen>,
    main_screen: Res<MainScreen>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| match screen.as_mut() {
        AppScreen::StartMenu(ctx) => f.render_stateful_widget(main_screen.as_ref(), f.size(), ctx),
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
        standalone_plugin::StandalonePlugin,
        tui_plugin::{AppScreen, TuiPlugin},
    };

    #[test]
    fn test_change_screen() {
        let mut app = App::new();
        app.add_plugins((StandalonePlugin::default(), TuiPlugin::testing()));
        app.update();
        app.update();
        let world = &mut app.world;
        assert!(matches!(
            *world.resource::<AppScreen>(),
            AppScreen::Explorer(_)
        ));
    }
}