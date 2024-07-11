use bevy::prelude::*;
use crossterm::event::KeyEventKind;
use ratatui::widgets::{List, ListState, StatefulWidget};

use crate::{
    core_plugin::CoreEvent,
    keyboard::StartMenuKeymap,
    utils::{
        list::{select_next_clamp, select_previous_clamp},
        ui::Direction2,
    },
};

use super::{AppScreen, ChangeAppScreen, ScreenContext};

const SCREENS: [(ChangeAppScreen, &str); 3] = [
    (ChangeAppScreen::Singleplayer, "Singleplayer"),
    (ChangeAppScreen::Multiplayer, "Multiplayer"),
    (ChangeAppScreen::Explorer, "Explore"),
];

pub struct StartMenuPlugin;

#[derive(Event)]
pub enum StartMenuEvent {
    Quit,
    Select(Direction2),
}

pub struct StartMenuContext {
    list_state: ListState,
}

pub struct StartMenu;

impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartMenuEvent>()
            .add_systems(Update, handle_start_menu_events);
    }
}

impl ScreenContext for StartMenuContext {
    type ScreenEvent = StartMenuEvent;

    type ScreenKeymap = StartMenuKeymap;

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
        use StartMenuEvent::*;

        internal_event.send(match key_event {
            e if keymap.select_next.matches(e) => Select(Down),
            e if keymap.select_previous.matches(e) => Select(Up),
            e if keymap.quit.matches(e) => Quit,
            e if keymap.validate.matches(e) => return Some(self.get_next_screen()),
            _ => return None,
        });
        None
    }
}
impl StartMenuContext {
    fn get_next_screen(&self) -> ChangeAppScreen {
        match self.list_state.selected().unwrap() {
            i if i < SCREENS.len() => SCREENS[i].0,
            _ => unreachable!(),
        }
    }
}

impl Default for StartMenuContext {
    fn default() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
        }
    }
}

pub fn handle_start_menu_events(
    mut screen: ResMut<AppScreen>,
    mut events: EventReader<StartMenuEvent>,
    mut core_events: EventWriter<CoreEvent>,
) {
    if let AppScreen::StartMenu(context) = screen.as_mut() {
        for event in events.read() {
            match event {
                StartMenuEvent::Quit => {
                    core_events.send(CoreEvent::Quit);
                }
                StartMenuEvent::Select(d) => match d {
                    Direction2::Down => {
                        select_next_clamp(&mut context.list_state, SCREENS.len() - 1)
                    }
                    Direction2::Up => select_previous_clamp(&mut context.list_state, 0),
                },
            }
        }
    }
}

impl StatefulWidget for StartMenu {
    type State = StartMenuContext;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let (_, entries): (Vec<_>, Vec<&str>) = SCREENS.into_iter().unzip();
        List::new(entries)
            .highlight_symbol(">")
            .render(area, buf, &mut state.list_state);
    }
}
