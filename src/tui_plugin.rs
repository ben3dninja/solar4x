use bevy::prelude::*;
use bevy_ratatui::{
    error::exit_on_error, event::KeyEvent, terminal::RatatuiContext, RatatuiPlugins,
};
use explorer_screen::ExplorerPlugin;

use crate::{
    core_plugin::{BodyInfo, GameSet, PrimaryBody},
    engine_plugin::Position,
    keyboard::Keymap,
};

use self::explorer_screen::{ExplorerContext, ExplorerEvent, ExplorerScreen};

pub mod explorer_screen;
pub mod info_plugin;
pub mod search_plugin;
pub mod space_map_plugin;
pub mod tree_plugin;

#[derive(Default)]
pub struct TuiPlugin {
    headless: bool,
    start_in_explorer: bool,
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
                .add_systems(PostUpdate, render.pipe(exit_on_error).in_set(GameSet))
                .add_systems(
                    PreUpdate,
                    handle_input.before(change_screen).in_set(GameSet),
                );
        }
        app.add_plugins(ExplorerPlugin)
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

#[derive(Resource, Default)]
pub enum AppScreen {
    #[default]
    StartMenu,
    Explorer(ExplorerContext),
}

#[derive(Event)]
pub enum ChangeAppScreen {
    StartMenu,
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
    keymap: Res<Keymap>,
) {
    match screen.as_mut() {
        AppScreen::StartMenu => todo!(),
        AppScreen::Explorer(ctx) => key_reader.read().for_each(|e| {
            if let Some(s) = ctx.read_input(e, keymap.as_ref(), explorer_events.as_mut()) {
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
            ChangeAppScreen::StartMenu => AppScreen::StartMenu,
            ChangeAppScreen::Explorer => {
                AppScreen::Explorer(ExplorerContext::new(primary.single(), &bodies))
            }
        }
    }
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    mut screen: ResMut<AppScreen>,
    explorer_screen: Res<ExplorerScreen>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| match screen.as_mut() {
        AppScreen::StartMenu => todo!(),
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
