use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent as CKeyEvent;
use crossterm::event::KeyEventKind;

use crate::core_plugin::CoreEvent;
use crate::engine_plugin::EngineEvent;
use crate::ui_plugin::search_plugin::SearchViewEvent;
use crate::ui_plugin::space_map_plugin::SpaceMapEvent;
use crate::ui_plugin::tree_plugin::TreeViewEvent;
use crate::ui_plugin::WindowEvent;
use crate::{keyboard::Keymap, ui_plugin::FocusView};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Keymap::default()).add_systems(
            PreUpdate,
            (
                (send_tree_events, send_space_map_events).run_if(resource_equals(FocusView::Tree)),
                send_search_events.run_if(resource_equals(FocusView::Search)),
                send_window_events,
                send_core_events,
                send_engine_events,
            ),
        );
    }
}

fn send_tree_events(
    mut keys: EventReader<KeyEvent>,
    mut writer: EventWriter<TreeViewEvent>,
    keymap: Res<Keymap>,
) {
    use crate::utils::ui::Direction2::*;
    use TreeViewEvent::*;
    let codes = &keymap.tree;
    for event in keys.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        writer.send(match event {
            e if codes.select_next.matches(e) => SelectTree(Down),
            e if codes.select_previous.matches(e) => SelectTree(Up),
            e if codes.toggle_expand.matches(e) => ToggleTreeExpansion,
            _ => continue,
        });
    }
}
fn send_space_map_events(
    mut keys: EventReader<KeyEvent>,
    mut writer: EventWriter<SpaceMapEvent>,
    keymap: Res<Keymap>,
) {
    use crate::utils::ui::Direction2::*;
    use crate::utils::ui::Direction4::*;
    use SpaceMapEvent::*;
    let codes = &keymap.tree;
    for event in keys.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        writer.send(match event {
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
    mut writer: EventWriter<SearchViewEvent>,
    keymap: Res<Keymap>,
) {
    use crate::utils::ui::Direction2::*;
    use SearchViewEvent::*;
    let codes = &keymap.search;
    for event in keys.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        writer.send(match event {
            e if codes.delete_char.matches(e) => DeleteChar,
            e if codes.validate_search.matches(e) => ValidateSearch,
            e if codes.move_cursor_left.matches(e) => MoveCursor(Down),
            e if codes.move_cursor_right.matches(e) => MoveCursor(Up),
            e if codes.select_next.matches(e) => SelectSearch(Down),
            e if codes.select_previous.matches(e) => SelectSearch(Up),
            KeyEvent(CKeyEvent {
                code: KeyCode::Char(char),
                ..
            }) => WriteChar(*char),
            _ => continue,
        });
    }
}

fn send_window_events(
    mut keys: EventReader<KeyEvent>,
    mut tree_writer: EventWriter<WindowEvent>,
    keymap: Res<Keymap>,
    focus_view: Res<FocusView>,
) {
    use FocusView::*;
    use WindowEvent::*;
    for event in keys.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        match *focus_view {
            Tree => {
                let codes = &keymap.tree;
                tree_writer.send(ChangeFocus(match event {
                    e if codes.enter_search.matches(e) => Search,
                    e if codes.display_info.matches(e) => Info,
                    _ => continue,
                }));
            }
            Search => {
                let codes = &keymap.search;
                tree_writer.send(ChangeFocus(match event {
                    e if codes.leave_search.matches(e) => Search,
                    _ => continue,
                }));
            }
            Info => {
                let codes = &keymap.info;
                tree_writer.send(ChangeFocus(match event {
                    e if codes.leave_info.matches(e) => Info,
                    _ => continue,
                }));
            }
        }
    }
}

fn send_core_events(
    mut keys: EventReader<KeyEvent>,
    mut core_writer: EventWriter<CoreEvent>,
    keymap: Res<Keymap>,
    focus_view: Res<FocusView>,
) {
    use CoreEvent::*;
    use FocusView::*;
    for event in keys.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        match *focus_view {
            Tree => {
                let codes = &keymap.tree;
                core_writer.send(match event {
                    e if codes.quit.matches(e) => Quit,
                    _ => continue,
                });
            }
            _ => continue,
        }
    }
}

fn send_engine_events(
    mut keys: EventReader<KeyEvent>,
    mut core_writer: EventWriter<EngineEvent>,
    keymap: Res<Keymap>,
    focus_view: Res<FocusView>,
) {
    use crate::utils::ui::Direction2::*;
    use EngineEvent::*;
    use FocusView::*;
    for event in keys.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        match *focus_view {
            Tree => {
                let codes = &keymap.tree;
                core_writer.send(match event {
                    e if codes.speed_up.matches(e) => EngineSpeed(Up),
                    e if codes.slow_down.matches(e) => EngineSpeed(Down),
                    e if codes.toggle_time.matches(e) => ToggleTime,
                    _ => continue,
                });
            }
            _ => continue,
        }
    }
}
