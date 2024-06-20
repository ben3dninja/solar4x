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
                    match event.code {
                        c if c == codes.quit => return Ok(AppMessage::Quit),
                        c if c == codes.speed_up => {
                            self.engine.speed *= 1.5;
                        }
                        c if c == codes.slow_down => {
                            self.engine.speed /= 1.5;
                        }
                        c if c == codes.toggle_time => self.toggle_time_switch(),
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
                    match event.code {
                        c if c == codes.select_next => SelectTree(Down),
                        c if c == codes.select_previous => SelectTree(Up),
                        c if c == codes.zoom_in => Zoom(Up),
                        c if c == codes.zoom_out => Zoom(Down),
                        c if c == codes.display_info => BodyInfo,
                        c if c == codes.toggle_expand => ToggleTreeExpansion,
                        c if c == codes.map_offset_up => MapOffset(Front),
                        c if c == codes.map_offset_left => MapOffset(Left),
                        c if c == codes.map_offset_down => MapOffset(Back),
                        c if c == codes.map_offset_right => MapOffset(Right),
                        c if c == codes.map_offset_reset => MapOffsetReset,
                        c if c == codes.enter_search => EnterSearchView,
                        c if c == codes.focus => FocusBody,
                        c if c == codes.autoscale => Autoscale,
                        _ => return Ok(()),
                    }
                }),
                ExplorerMode::Search => UiEvent::Search({
                    let codes = &self.keymap.search;
                    match event.code {
                        c if c == codes.delete_char => DeleteChar,
                        c if c == codes.validate_search => ValidateSearch,
                        c if c == codes.move_cursor_left => MoveCursor(Down),
                        c if c == codes.move_cursor_right => MoveCursor(Up),
                        c if c == codes.select_next => SelectSearch(Down),
                        c if c == codes.select_previous => SelectSearch(Up),
                        c if c == codes.leave_search => LeaveSearchView,
                        KeyCode::Char(char) => WriteChar(char),
                        _ => return Ok(()),
                    }
                }),
            },
            AppScreen::Info => UiEvent::Info({
                let codes = &self.keymap.info;
                match event.code {
                    c if c == codes.leave_info => LeaveInfoView,
                    _ => return Ok(()),
                }
            }),
        })
    }
}
