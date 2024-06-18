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
                    match event.code {
                        KeyCode::Esc => return Ok(AppMessage::Quit),
                        KeyCode::Char('>') => {
                            self.engine.speed *= 1.5;
                        }
                        KeyCode::Char('<') => {
                            self.engine.speed /= 1.5;
                        }
                        KeyCode::Char('t') => self.toggle_time_switch(),
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
        let (w, a) = ('w', 'a');
        #[cfg(feature = "azerty")]
        let (w, a) = ('z', 'q');
        self.ui_event_sender.send(match self.get_current_screen() {
            AppScreen::Main => match self.get_explorer_mode() {
                ExplorerMode::Tree => UiEvent::Tree(match event.code {
                    KeyCode::Down => SelectTree(Down),
                    KeyCode::Up => SelectTree(Up),
                    KeyCode::Char('+') => Zoom(Up),
                    KeyCode::Char('-') => Zoom(Down),
                    KeyCode::Char('i') => BodyInfo,
                    KeyCode::Char(' ') => ToggleTreeExpansion,
                    KeyCode::Char(c) if c == w => MapOffset(Front),
                    KeyCode::Char(c) if c == a => MapOffset(Left),
                    KeyCode::Char('s') => MapOffset(Back),
                    KeyCode::Char('d') => MapOffset(Right),
                    KeyCode::Char('0') => MapOffsetReset,
                    KeyCode::Char('/') => EnterSearchView,
                    KeyCode::Char('f') => FocusBody,
                    KeyCode::Char('x') => Autoscale,
                    _ => return Ok(()),
                }),
                ExplorerMode::Search => UiEvent::Search(match event.code {
                    KeyCode::Backspace => DeleteChar,
                    KeyCode::Enter => ValidateSearch,
                    KeyCode::Left => MoveCursor(Down),
                    KeyCode::Right => MoveCursor(Up),
                    KeyCode::Down => SelectSearch(Down),
                    KeyCode::Up => SelectSearch(Up),
                    KeyCode::Esc => LeaveSearchView,
                    KeyCode::Char(char) => WriteChar(char),
                    _ => return Ok(()),
                }),
            },
            AppScreen::Info => UiEvent::Info(match event.code {
                KeyCode::Char('i') => LeaveInfoView,
                _ => return Ok(()),
            }),
        })
    }
}
