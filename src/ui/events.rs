use nalgebra::Vector2;

use crate::{
    app::AppMessage,
    utils::ui::{Direction2, Direction4},
};

use super::{AppScreen, UiState, OFFSET_STEP};

#[derive(Debug)]
pub enum TreeViewEvent {
    SelectTree(Direction2),
    Zoom(Direction2),
    BodyInfo,
    ToggleTreeExpansion,
    MapOffset(Direction4),
    MapOffsetReset,
    EnterSearchView,
    FocusBody,
    Autoscale,
}

#[derive(Debug)]
pub enum SearchViewEvent {
    MoveCursor(Direction2),
    SelectSearch(Direction2),
    LeaveSearchView,
    ValidateSearch,
    WriteChar(char),
    DeleteChar,
}

#[derive(Debug)]
pub enum InfoViewEvent {
    LeaveInfoView,
}

#[derive(Debug)]
pub enum UiEvent {
    Tree(TreeViewEvent),
    Search(SearchViewEvent),
    Info(InfoViewEvent),
    Quit,
}

impl UiState {
    pub fn handle_events(&mut self) -> AppMessage {
        while let Ok(event) = self.ui_event_receiver.try_recv() {
            match event {
                UiEvent::Quit => return AppMessage::Quit,
                UiEvent::Search(e) => self.handle_search_event(e),
                UiEvent::Tree(e) => self.handle_tree_event(e),
                UiEvent::Info(e) => self.handle_info_event(e),
            }
        }
        AppMessage::Idle
    }

    pub fn handle_tree_event(&mut self, event: TreeViewEvent) {
        use Direction2::*;
        use Direction4::*;
        use TreeViewEvent::*;
        match event {
            SelectTree(d) => match d {
                Down => self.select_next_tree(),
                Up => self.select_previous_tree(),
            },
            Zoom(d) => match d {
                Down => self.zoom_level /= 1.5,
                Up => self.zoom_level *= 1.5,
            },
            BodyInfo => self.set_current_screen(AppScreen::Info),
            ToggleTreeExpansion => self.toggle_selection_expansion(),
            MapOffset(d) => {
                self.offset += (match d {
                    Front | Right => 1.,
                    _ => -1.,
                } * OFFSET_STEP as f64
                    / self.zoom_level)
                    .round() as i64
                    * match d {
                        Front | Back => Vector2::y(),
                        _ => Vector2::x(),
                    }
            }
            MapOffsetReset => self.offset = Vector2::zeros(),
            // KeyCode::Char('w') => {
            //     self.offset +=
            //         (OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::y()
            // }
            // KeyCode::Char('a') => {
            //     self.offset +=
            //         (-OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::x()
            // }
            // KeyCode::Char('s') => {
            //     self.offset +=
            //         (-OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::y()
            // }
            // KeyCode::Char('d') => {
            //     self.offset +=
            //         (OFFSET_STEP as f64 / self.zoom_level).round() as i64 * Vector2::x()
            // }
            EnterSearchView => self.enter_search_mode(),
            FocusBody => self.focus_body = self.selected_body_id_tree(),
            Autoscale => self.autoscale(),
        }
    }

    pub fn handle_search_event(&mut self, event: SearchViewEvent) {
        use Direction2::*;
        use SearchViewEvent::*;
        match event {
            DeleteChar => self.delete_char(),
            MoveCursor(d) => match d {
                Down => self.move_cursor_left(),
                Up => self.move_cursor_right(),
            },
            SelectSearch(d) => match d {
                Down => self.select_next_search(),
                Up => self.select_previous_search(),
            },
            LeaveSearchView => self.leave_search_mode(),
            ValidateSearch => self.validate_search(),
            WriteChar(char) => self.enter_char(char),
        }
    }

    pub fn handle_info_event(&mut self, event: InfoViewEvent) {
        use InfoViewEvent::*;
        match event {
            LeaveInfoView => self.set_current_screen(AppScreen::Main),
        }
    }
}
