mod search;
mod tree;

use std::{cell::RefCell, error::Error, io::Stdout, rc::Rc, time::Duration};

use crossterm::event::{self, KeyCode, KeyEventKind};
use fuzzy_matcher::skim::SkimMatcherV2;
use nalgebra::Vector2;
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};

use crate::bodies::{body_data::BodyType, body_id::BodyID, BodySystem};

use self::tree::TreeEntry;

type Tui = Terminal<CrosstermBackend<Stdout>>;

// frame rate in fps
const FRAME_RATE: f64 = 60.;
// Speed in days per second
const DEFAULT_SPEED: f64 = 10.;
const OFFSET_STEP: i64 = 1e8 as i64;

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

pub struct App {
    pub current_screen: AppScreen,
    pub explorer_mode: ExplorerMode,
    pub system: Rc<RefCell<BodySystem>>,
    // 1 represents the level where all the system is seen,
    // higher values mean more zoom
    pub zoom_level: f64,
    // 1 represents 1 day / second
    pub speed: f64,
    pub offset: Vector2<i64>,
    pub time_switch: bool,

    pub tree_entries: Vec<TreeEntry>,
    pub tree_state: ListState,
    pub search_entries: Vec<BodyID>,
    pub search_state: ListState,
    pub search_character_index: usize,
    pub search_input: String,
    pub search_matcher: SkimMatcherV2,
}

enum AppMessage {
    Quit,
    Idle,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let system = Rc::clone(&BodySystem::new_system_with_filter(|data| {
            matches!(
                data.body_type,
                BodyType::Star | BodyType::Planet | BodyType::Moon
            )
        })?);
        let search_entries: Vec<_> = system.borrow().bodies.keys().map(Clone::clone).collect();
        let main_body = system.borrow().primary_body_id().ok_or("No primary body")?;
        Ok(Self {
            current_screen: AppScreen::default(),
            explorer_mode: ExplorerMode::default(),
            tree_entries: vec![TreeEntry::new_main_body(main_body)],
            tree_state: ListState::default().with_selected(Some(0)),
            search_state: ListState::default().with_selected(Some(0)),
            system,
            zoom_level: 1.,
            speed: DEFAULT_SPEED,
            offset: Vector2::zeros(),
            time_switch: true,
            search_entries,
            search_character_index: 0,
            search_input: String::new(),
            search_matcher: SkimMatcherV2::default(),
        })
    }

    fn run_logic(&mut self) {
        match self.explorer_mode {
            ExplorerMode::Search => self.search_entries = self.search(&self.search_input),
            _ => {}
        }
        let mut system = self.system.borrow_mut();
        if self.time_switch {
            system.elapse_time(self.speed / FRAME_RATE);
            system.update_orbits();
        }
    }

    fn handle_events(&mut self) -> Result<AppMessage, Box<dyn Error>> {
        if event::poll(Duration::from_secs_f64(1. / FRAME_RATE))? {
            if let event::Event::Key(event) = event::read()? {
                if event.kind == KeyEventKind::Release {
                    return Ok(AppMessage::Idle);
                }
                match self.current_screen {
                    AppScreen::Main => match self.explorer_mode {
                        ExplorerMode::Tree => {
                            match event.code {
                                KeyCode::Esc => return Ok(AppMessage::Quit),
                                KeyCode::Down => self.select_next_tree(),
                                KeyCode::Up => self.select_previous_tree(),
                                KeyCode::Char('+') => {
                                    self.zoom_level *= 1.5;
                                }
                                KeyCode::Char('-') => {
                                    self.zoom_level /= 1.5;
                                }
                                KeyCode::Char('>') => {
                                    self.speed *= 1.5;
                                }
                                KeyCode::Char('<') => {
                                    self.speed /= 1.5;
                                }
                                KeyCode::Char('i') => self.current_screen = AppScreen::Info,
                                KeyCode::Char(' ') => self.toggle_selection_expansion()?,
                                KeyCode::Char('w') => {
                                    self.offset += (OFFSET_STEP as f64 / self.zoom_level).round()
                                        as i64
                                        * Vector2::y()
                                }
                                KeyCode::Char('a') => {
                                    self.offset += (-OFFSET_STEP as f64 / self.zoom_level).round()
                                        as i64
                                        * Vector2::x()
                                }
                                KeyCode::Char('s') => {
                                    self.offset += (-OFFSET_STEP as f64 / self.zoom_level).round()
                                        as i64
                                        * Vector2::y()
                                }
                                KeyCode::Char('d') => {
                                    self.offset += (OFFSET_STEP as f64 / self.zoom_level).round()
                                        as i64
                                        * Vector2::x()
                                }
                                KeyCode::Char('t') => self.toggle_time_switch(),
                                KeyCode::Char('/') => {
                                    self.explorer_mode = ExplorerMode::Search;
                                }
                                _ => {}
                            }
                            #[cfg(feature = "azerty")]
                            match event.code {
                                KeyCode::Char('z') => {
                                    self.offset += (OFFSET_STEP as f64 / self.zoom_level.round())
                                        as i64
                                        * Vector2::y()
                                }
                                KeyCode::Char('q') => {
                                    self.offset += (-OFFSET_STEP as f64 / self.zoom_level).round()
                                        as i64
                                        * Vector2::x()
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
                            KeyCode::Esc => self.explorer_mode = ExplorerMode::Tree,
                            KeyCode::Enter => self.validate_search(),
                            KeyCode::Char(char) => self.enter_char(char),
                            _ => {}
                        },
                    },
                    AppScreen::Info => match event.code {
                        KeyCode::Char('i') => self.current_screen = AppScreen::Main,
                        _ => (),
                    },
                }
            }
        }
        Ok(AppMessage::Idle)
    }

    pub fn run(&mut self, tui: &mut Tui) -> Result<(), Box<dyn Error>> {
        loop {
            self.run_logic();
            tui.draw(|frame| self.draw_ui(frame))?;
            if let Ok(AppMessage::Quit) = self.handle_events() {
                break;
            }
        }
        Ok(())
    }

    fn select_body(&mut self, id: &BodyID) {
        let ancestors = self.system.borrow().get_body_ancestors(id);
        for body_id in ancestors {
            self.expand_entry_by_id(&body_id)
        }
        self.tree_state
            .select(self.tree_entries.iter().position(|entry| &entry.id == id));
    }

    fn toggle_time_switch(&mut self) {
        self.time_switch = !self.time_switch
    }
}

#[cfg(test)]
mod tests {
    use super::{App, ExplorerMode};

    #[test]
    fn test_select_body() {
        let mut app = App::new().unwrap();
        app.select_body(&"terre".into());
        assert_eq!(app.selected_body_id_tree(), "terre".into())
    }

    #[test]
    fn test_search() {
        let mut app = App::new().unwrap();
        app.toggle_selection_expansion().unwrap();
        app.select_next_tree();
        app.explorer_mode = ExplorerMode::Search;
        app.search_input = "Moo".into();
        app.run_logic();
        app.validate_search();
        assert_eq!(app.selected_body_id_tree(), "lune".into())
    }
}
