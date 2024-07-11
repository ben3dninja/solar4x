use bevy::prelude::*;
use crossterm::event::KeyEventKind;
use ratatui::widgets::{List, ListState, StatefulWidget};

use crate::{
    core_plugin::UiInitSet,
    keyboard::FleetScreenKeymap,
    main_game::InGame,
    spaceship::{ShipID, ShipInfo},
    utils::ui::Direction2,
};

use super::{AppScreen, ChangeAppScreen, ScreenContext};
pub struct FleetScreenPlugin;

impl Plugin for FleetScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FleetScreenEvent>()
            .add_systems(Update, handle_fleet_events)
            .add_systems(OnEnter(InGame), change_screen_to_fleet.in_set(UiInitSet));
    }
}

#[derive(Event, Copy, Clone)]
pub enum FleetScreenEvent {
    Select(Direction2),
    EditTrajectory,
}

#[derive(Default)]
pub struct FleetContext {
    list_state: ListState,
    ships: Vec<ShipInfo>,
}

impl FleetContext {
    fn new(ships: impl Iterator<Item = ShipInfo>) -> Self {
        Self {
            ships: ships.collect(),
            ..Default::default()
        }
    }
    fn selected_ship_id(&self) -> Option<ShipID> {
        self.list_state.selected().map(|i| self.ships[i].id)
    }
}

pub struct FleetScreen;

impl ScreenContext for FleetContext {
    type ScreenEvent = FleetScreenEvent;

    type ScreenKeymap = FleetScreenKeymap;

    fn read_input(
        &mut self,
        key_event: &bevy_ratatui::event::KeyEvent,
        keymap: &Self::ScreenKeymap,
        internal_event: &mut Events<Self::ScreenEvent>,
    ) -> Option<ChangeAppScreen> {
        if key_event.kind == KeyEventKind::Release {
            return None;
        }
        use Direction2::*;
        use FleetScreenEvent::*;

        internal_event.send(match key_event {
            e if keymap.select_next.matches(e) => Select(Down),
            e if keymap.select_previous.matches(e) => Select(Up),
            e if keymap.back.matches(e) => return Some(ChangeAppScreen::StartMenu),
            e if keymap.edit_trajectory.matches(e) => {
                if let Some(id) = self.selected_ship_id() {
                    return Some(ChangeAppScreen::TrajectoryEditor(id));
                } else {
                    return None;
                }
            }
            _ => return None,
        });
        None
    }
}

fn change_screen_to_fleet(mut screen: ResMut<AppScreen>, ships: Query<&ShipInfo>) {
    *screen = AppScreen::Fleet(FleetContext::new(ships.iter().cloned()));
}

fn handle_fleet_events() {}

impl StatefulWidget for FleetScreen {
    type State = FleetContext;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let entries = state.ships.iter().map(|s| s.id.to_string());
        List::new(entries)
            .highlight_symbol(">")
            .render(area, buf, &mut state.list_state);
    }
}
