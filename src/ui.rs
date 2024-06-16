mod draw;
pub mod events;
mod search;
mod tree;

use std::{
    io::{stdout, Result, Stdout},
    sync::{Arc, Mutex},
};

use crate::{
    app::body_id::BodyID,
    app::{info::SystemInfo, GlobalMap},
};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use fuzzy_matcher::skim::SkimMatcherV2;
use nalgebra::Vector2;
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};

use self::tree::TreeEntry;

const OFFSET_STEP: i64 = 1e8 as i64;

pub struct UiState {
    pub current_screen: AppScreen,
    pub explorer_mode: ExplorerMode,
    // 1 represents the level where all the system is seen,
    // higher values mean more zoom
    zoom_level: f64,
    offset: Vector2<i64>,
    focus_body: BodyID,
    tree_entries: Vec<TreeEntry>,
    tree_state: ListState,
    search_entries: Vec<BodyID>,
    search_state: ListState,
    search_character_index: usize,
    search_input: String,
    search_matcher: SkimMatcherV2,
    shared_info: Arc<SystemInfo>,
    pub global_map: Arc<Mutex<GlobalMap>>,
}

#[derive(Default)]
pub enum AppScreen {
    #[default]
    Main,
    Info,
}

#[derive(Default)]
pub enum ExplorerMode {
    #[default]
    Tree,
    Search,
}

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

impl UiState {
    pub fn new(shared_info: Arc<SystemInfo>, global_map: Arc<Mutex<GlobalMap>>) -> Result<Self> {
        let search_entries: Vec<BodyID> = shared_info.bodies.keys().cloned().collect();
        let main_body = shared_info.primary_body;
        Ok(Self {
            current_screen: AppScreen::default(),
            explorer_mode: ExplorerMode::default(),
            tree_entries: vec![TreeEntry::new_main_body(main_body)],
            tree_state: ListState::default().with_selected(Some(0)),
            search_state: ListState::default().with_selected(Some(0)),
            zoom_level: 1.,
            offset: Vector2::zeros(),
            focus_body: main_body,
            search_entries,
            search_character_index: 0,
            search_input: String::new(),
            search_matcher: SkimMatcherV2::default(),
            shared_info,
            global_map,
        })
    }

    pub fn update_search_selection(&mut self) {
        match self.explorer_mode {
            ExplorerMode::Search => self.search_entries = self.search(&self.search_input),
            _ => {}
        }
        if self.search_state.selected().is_none() && !self.search_entries.is_empty() {
            self.search_state.select(Some(0));
        }
    }

    pub fn render(&mut self, tui: &mut Tui) -> Result<()> {
        self.update_search_selection();
        tui.draw(|frame| self.draw_ui(frame))?;
        Ok(())
    }

    pub fn setup_tui() -> Result<Tui> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Terminal::new(CrosstermBackend::new(stdout()))
    }

    pub fn reset_tui() -> Result<()> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn select_body(&mut self, id: BodyID) {
        let ancestors = self.shared_info.get_body_ancestors(id);
        for body_id in ancestors {
            self.expand_entry_by_id(body_id);
        }
        self.tree_state
            .select(self.tree_entries.iter().position(|entry| entry.id == id));
    }
    fn autoscale(&mut self) {
        let bodies = &self.shared_info.bodies;
        if let Some(body) = bodies.get(&self.focus_body) {
            if let Some(max_dist) = body
                .orbiting_bodies
                .iter()
                .map(|id| bodies.get(id).map_or(0, |body| body.semimajor_axis))
                .max()
            {
                self.zoom_level = self.shared_info.get_max_distance() as f64 / (max_dist as f64);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::App;

    #[test]
    fn test_select_body() {
        let mut app = App::new_simple(true).unwrap();
        app.ui.select_body("terre".into());
        assert_eq!(app.ui.selected_body_id_tree(), "terre".into())
    }
}
