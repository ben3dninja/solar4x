use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::widgets::{List, ListState, StatefulWidget};

use crate::{
    client_plugin::ClientMode,
    core_plugin::CoreEvent,
    keyboard::Keymap,
    utils::{list::ClampedList, ui::Direction2},
};

use super::AppScreen;

const SCREENS: [(ClientMode, &str); 3] = [
    (ClientMode::Singleplayer, "Singleplayer"),
    (ClientMode::Multiplayer, "Multiplayer"),
    (ClientMode::Explorer, "Explore"),
];

pub struct StartMenuPlugin;

#[derive(Event)]
pub enum StartMenuEvent {
    Quit,
    Select(Direction2),
    Validate,
}

#[derive(Resource)]
pub struct StartMenuContext {
    list_state: ListState,
}

pub struct StartMenu;

impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StartMenuContext::default())
            .add_event::<StartMenuEvent>()
            .add_systems(
                Update,
                (read_input, handle_events)
                    .chain()
                    .run_if(in_state(AppScreen::StartMenu)),
            )
            .add_systems(OnEnter(ClientMode::None), create_screen);
    }
}

fn create_screen(mut next_screen: ResMut<NextState<AppScreen>>) {
    next_screen.set(AppScreen::StartMenu);
}

fn read_input(
    mut key_event: EventReader<KeyEvent>,
    keymap: Res<Keymap>,
    mut internal_event: EventWriter<StartMenuEvent>,
) {
    for KeyEvent(event) in key_event.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        use Direction2::*;
        use StartMenuEvent::*;

        let keymap = &keymap.start_menu;
        internal_event.send(match event {
            e if keymap.select_next.matches(e) => Select(Down),
            e if keymap.select_previous.matches(e) => Select(Up),
            e if keymap.quit.matches(e) => Quit,
            e if keymap.validate.matches(e) => Validate,
            _ => return,
        });
    }
}

impl StartMenuContext {
    fn get_next_mode(&self) -> ClientMode {
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

impl ClampedList for StartMenuContext {
    fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    fn len(&self) -> usize {
        SCREENS.len()
    }
}

pub fn handle_events(
    mut next_mode: ResMut<NextState<ClientMode>>,
    mut context: ResMut<StartMenuContext>,
    mut events: EventReader<StartMenuEvent>,
    mut core_events: EventWriter<CoreEvent>,
) {
    for event in events.read() {
        match event {
            StartMenuEvent::Quit => {
                core_events.send(CoreEvent::Quit);
            }
            StartMenuEvent::Select(d) => context.select_adjacent(*d),
            StartMenuEvent::Validate => next_mode.set(context.get_next_mode()),
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
