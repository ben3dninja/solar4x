use std::{error::Error, sync::mpsc::SendError, time::Duration};

use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};

use crate::ui::{events::UiEvent, AppScreen, ExplorerMode};

use super::{App, AppMessage};

impl App {
    pub fn handle_input(&mut self) -> Result<AppMessage, Box<dyn Error>> {
        if event::poll(Duration::ZERO)? {
            if let event::Event::Key(event) = event::read()? {
                if event.kind == KeyEventKind::Release {
                    return Ok(AppMessage::Idle);
                }
                if matches!(
                    (self.get_current_screen(), self.get_explorer_mode()),
                    (AppScreen::Main, ExplorerMode::Tree)
                ) {
                    let codes = &self.keymap.tree;
                    match &event {
                        e if codes.quit.matches(e) => return Ok(AppMessage::Quit),
                        e if codes.speed_up.matches(e) => {
                            self.engine.speed *= 1.5;
                        }
                        e if codes.slow_down.matches(e) => {
                            self.engine.speed /= 1.5;
                        }
                        e if codes.toggle_time.matches(e) => self.toggle_time_switch(),
                        _ => (),
                    }
                }
                self.send_ui_event(&event)?;
            }
        }
        Ok(AppMessage::Idle)
    }

    fn send_ui_event(&self, event: &KeyEvent) -> Result<(), SendError<UiEvent>> {
        use crate::ui::events::InfoViewEvent::*;
        use crate::ui::events::SearchViewEvent::*;
        use crate::ui::events::TreeViewEvent::*;
        use crate::utils::ui::Direction2::*;
        use crate::utils::ui::Direction4::*;
        self.ui_event_sender.send(match self.get_current_screen() {
            AppScreen::Main => match self.get_explorer_mode() {
                ExplorerMode::Tree => UiEvent::Tree({
                    let codes = &self.keymap.tree;
                    match event {
                        e if codes.select_next.matches(e) => SelectTree(Down),
                        e if codes.select_previous.matches(e) => SelectTree(Up),
                        e if codes.zoom_in.matches(e) => Zoom(Up),
                        e if codes.zoom_out.matches(e) => Zoom(Down),
                        e if codes.display_info.matches(e) => BodyInfo,
                        e if codes.toggle_expand.matches(e) => ToggleTreeExpansion,
                        e if codes.map_offset_up.matches(e) => MapOffset(Front),
                        e if codes.map_offset_left.matches(e) => MapOffset(Left),
                        e if codes.map_offset_down.matches(e) => MapOffset(Back),
                        e if codes.map_offset_right.matches(e) => MapOffset(Right),
                        e if codes.map_offset_reset.matches(e) => MapOffsetReset,
                        e if codes.enter_search.matches(e) => EnterSearchView,
                        e if codes.focus.matches(e) => FocusBody,
                        e if codes.autoscale.matches(e) => Autoscale,
                        _ => return Ok(()),
                    }
                }),
                ExplorerMode::Search => UiEvent::Search({
                    let codes = &self.keymap.search;
                    match event {
                        e if codes.delete_char.matches(e) => DeleteChar,
                        e if codes.validate_search.matches(e) => ValidateSearch,
                        e if codes.move_cursor_left.matches(e) => MoveCursor(Down),
                        e if codes.move_cursor_right.matches(e) => MoveCursor(Up),
                        e if codes.select_next.matches(e) => SelectSearch(Down),
                        e if codes.select_previous.matches(e) => SelectSearch(Up),
                        e if codes.leave_search.matches(e) => LeaveSearchView,
                        KeyEvent {
                            code: KeyCode::Char(char),
                            ..
                        } => WriteChar(*char),
                        _ => return Ok(()),
                    }
                }),
            },
            AppScreen::Info => UiEvent::Info({
                let codes = &self.keymap.info;
                match event {
                    e if codes.leave_info.matches(e) => LeaveInfoView,
                    _ => return Ok(()),
                }
            }),
        })
    }
}
