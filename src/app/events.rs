use std::{error::Error, time::Duration};

use crossterm::event::{self, KeyCode, KeyEventKind};

use crate::ui::{AppScreen, ExplorerMode};

use super::{App, FRAME_RATE};

pub enum AppMessage {
    Quit,
    Idle,
}
impl App {
    pub fn handle_events(&mut self) -> Result<AppMessage, Box<dyn Error>> {
        if event::poll(Duration::from_secs_f64(1. / FRAME_RATE))? {
            if let event::Event::Key(event) = event::read()? {
                if event.kind == KeyEventKind::Release {
                    return Ok(AppMessage::Idle);
                }
                let ui = &self.ui;
                match ui.current_screen {
                    AppScreen::Main => {
                        match ui.explorer_mode {
                            ExplorerMode::Tree => match event.code {
                                KeyCode::Esc => return Ok(AppMessage::Quit),
                                KeyCode::Char('>') => {
                                    self.engine.speed *= 1.5;
                                }
                                KeyCode::Char('<') => {
                                    self.engine.speed /= 1.5;
                                }
                                KeyCode::Char('t') => self.toggle_time_switch(),
                                _ => (),
                            },
                            _ => (),
                        }
                        self.ui.handle_main_screen_events(&event)?;
                    }
                    AppScreen::Info => self.ui.handle_info_events(&event),
                }
            }
        }
        Ok(AppMessage::Idle)
    }
}
