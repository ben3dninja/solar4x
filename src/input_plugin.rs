use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent as CKeyEvent;

use crate::{
    keyboard::Keymap,
    ui_plugin::{AppScreen, ExplorerMode},
    utils::ui::{Direction2, Direction4},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Keymap::default()).add_systems(
            PreUpdate,
            (
                (
                    (send_tree_events, send_space_map_events)
                        .run_if(resource_equals(ExplorerMode::Tree)),
                    send_search_events,
                )
                    .run_if(resource_equals(AppScreen::Main)),
                send_info_events.run_if(resource_equals(AppScreen::Info)),
            ),
        );
    }
}
#[derive(Debug, Event)]
pub enum TreeViewEvent {
    SelectTree(Direction2),
    BodyInfo,
    ToggleTreeExpansion,
    EnterSearchView,
}

#[derive(Debug, Event)]
pub enum SpaceMapEvent {
    Zoom(Direction2),
    MapOffset(Direction4),
    MapOffsetReset,
    FocusBody,
    Autoscale,
}

#[derive(Debug, Event)]
pub enum SearchViewEvent {
    MoveCursor(Direction2),
    SelectSearch(Direction2),
    LeaveSearchView,
    ValidateSearch,
    WriteChar(char),
    DeleteChar,
}

#[derive(Debug, Event)]
pub enum InfoViewEvent {
    LeaveInfoView,
}

fn send_tree_events(
    mut keys: EventReader<KeyEvent>,
    mut tree_writer: EventWriter<TreeViewEvent>,
    keymap: Res<Keymap>,
) {
    use crate::utils::ui::Direction2::*;
    use TreeViewEvent::*;
    let codes = &keymap.tree;
    for event in keys.read() {
        tree_writer.send(match event {
            e if codes.select_next.matches(e) => SelectTree(Down),
            e if codes.select_previous.matches(e) => SelectTree(Up),
            e if codes.display_info.matches(e) => BodyInfo,
            e if codes.toggle_expand.matches(e) => ToggleTreeExpansion,
            e if codes.enter_search.matches(e) => EnterSearchView,
            _ => continue,
        });
    }
}
fn send_space_map_events(
    mut keys: EventReader<KeyEvent>,
    mut tree_writer: EventWriter<SpaceMapEvent>,
    keymap: Res<Keymap>,
) {
    use crate::utils::ui::Direction2::*;
    use crate::utils::ui::Direction4::*;
    use SpaceMapEvent::*;
    let codes = &keymap.tree;
    for event in keys.read() {
        tree_writer.send(match event {
            e if codes.zoom_in.matches(e) => Zoom(Up),
            e if codes.zoom_out.matches(e) => Zoom(Down),
            e if codes.map_offset_up.matches(e) => MapOffset(Front),
            e if codes.map_offset_left.matches(e) => MapOffset(Left),
            e if codes.map_offset_down.matches(e) => MapOffset(Back),
            e if codes.map_offset_right.matches(e) => MapOffset(Right),
            e if codes.map_offset_reset.matches(e) => MapOffsetReset,
            e if codes.focus.matches(e) => FocusBody,
            e if codes.autoscale.matches(e) => Autoscale,
            _ => continue,
        });
    }
}
fn send_search_events(
    mut keys: EventReader<KeyEvent>,
    mut tree_writer: EventWriter<SearchViewEvent>,
    keymap: Res<Keymap>,
) {
    use crate::utils::ui::Direction2::*;
    use SearchViewEvent::*;
    let codes = &keymap.search;
    for event in keys.read() {
        tree_writer.send(match event {
            e if codes.delete_char.matches(e) => DeleteChar,
            e if codes.validate_search.matches(e) => ValidateSearch,
            e if codes.move_cursor_left.matches(e) => MoveCursor(Down),
            e if codes.move_cursor_right.matches(e) => MoveCursor(Up),
            e if codes.select_next.matches(e) => SelectSearch(Down),
            e if codes.select_previous.matches(e) => SelectSearch(Up),
            e if codes.leave_search.matches(e) => LeaveSearchView,
            KeyEvent(CKeyEvent {
                code: KeyCode::Char(char),
                ..
            }) => WriteChar(*char),
            _ => continue,
        });
    }
}
fn send_info_events(
    mut keys: EventReader<KeyEvent>,
    mut tree_writer: EventWriter<InfoViewEvent>,
    keymap: Res<Keymap>,
) {
    use InfoViewEvent::*;
    let codes = &keymap.info;
    for event in keys.read() {
        tree_writer.send(match event {
            e if codes.leave_info.matches(e) => LeaveInfoView,
            _ => continue,
        });
    }
}
