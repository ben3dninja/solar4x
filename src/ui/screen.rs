use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use editor::{EditorContext, EditorScreen};
use explorer::{ExplorerContext, ExplorerScreen};
use fleet::{FleetContext, FleetScreen};
use start::{StartMenu, StartMenuContext};

use crate::objects::ships::ShipID;

use super::widget::space_map::SpaceMap;

mod editor;
mod explorer;
mod fleet;
mod start;

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

pub fn plugin(app: &mut App) {
    app.add_plugins((
        StartMenuPlugin,
        ExplorerScreenPlugin,
        FleetScreenPlugin,
        EditorPlugin,
    ))
    .init_state::<AppScreen>()
    .init_resource::<PreviousScreen>();
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
