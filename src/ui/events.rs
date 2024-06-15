use std::error::Error;

use crossterm::event::{KeyCode, KeyEvent};
use nalgebra::Vector2;

use super::{AppScreen, ExplorerMode, UiState, OFFSET_STEP};

impl UiState {
    pub fn handle_main_screen_events(&mut self, event: &KeyEvent) -> Result<(), Box<dyn Error>> {
        match self.explorer_mode {
            ExplorerMode::Tree => {
                match event.code {
                    KeyCode::Down => self.select_next_tree(),
                    KeyCode::Up => self.select_previous_tree(),
                    KeyCode::Char('+') => {
                        self.zoom_level *= 1.5;
                    }
                    KeyCode::Char('-') => {
                        self.zoom_level /= 1.5;
                        self.explorer_mode = ExplorerMode::Tree
                    }
                    KeyCode::Char('i') => self.current_screen = AppScreen::Info,
                    KeyCode::Char(' ') => self.toggle_selection_expansion()?,
                    KeyCode::Char('w') => {
                        self.offset +=
                            (OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::y()
                    }
                    KeyCode::Char('a') => {
                        self.offset +=
                            (-OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::x()
                    }
                    KeyCode::Char('s') => {
                        self.offset +=
                            (-OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::y()
                    }
                    KeyCode::Char('d') => {
                        self.offset +=
                            (OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::x()
                    }
                    KeyCode::Char('/') => self.enter_search_mode(),
                    KeyCode::Char('f') => self.focus_body = self.selected_body_id_tree(),
                    KeyCode::Char('x') => self.autoscale(),
                    _ => {}
                }
                #[cfg(feature = "azerty")]
                match event.code {
                    KeyCode::Char('z') => {
                        self.offset +=
                            (OFFSET_STEP as f64 / self.zoom_level.round()) as i64 * Vector2::y()
                    }
                    KeyCode::Char('q') => {
                        self.offset +=
                            (-OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::x()
                    }
                    _ => {}
                }
            }
            ExplorerMode::Search => match event.code {
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Down => self.select_next_search(),
                KeyCode::Up => self.select_previous_search(),
                KeyCode::Esc => self.leave_search_mode(),
                KeyCode::Enter => self.validate_search(),
                KeyCode::Char(char) => self.enter_char(char),
                _ => {}
            },
        }
        Ok(())
    }

    pub fn handle_info_events(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char('i') => self.current_screen = AppScreen::Main,
            _ => (),
        }
    }
}
