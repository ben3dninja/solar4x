mod list;
use std::{cell::RefCell, error::Error, io::Stdout, rc::Rc, time::Duration};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};

use crate::bodies::{body_id::BodyID, BodySystem};

use self::list::ListEntry;

type Tui = Terminal<CrosstermBackend<Stdout>>;

// frame rate in fps
const FRAME_RATE: f64 = 60.;
const DEFAULT_SPEED: f64 = 10.;

pub enum AppScreen {
    Main,
    Info,
}

pub struct App {
    pub current_screen: AppScreen,
    pub main_body: BodyID,
    pub system: Rc<RefCell<BodySystem>>,
    pub list_mapping: Vec<BodyID>,
    pub list_state: ListState,
    pub listed_bodies: Vec<ListEntry>,
    // 1 represents the level where all the system is seen,
    // higher values mean more zoom
    pub zoom_level: f64,
    // 1 represents 1 day / second
    pub speed: f64,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let system = Rc::clone(&BodySystem::simple_solar_system()?);
        let list_mapping = system.borrow().bodies_by_distance();
        let main_body = system.borrow().primary_body_id().ok_or("No primary body")?;
        Ok(Self {
            current_screen: AppScreen::Main,
            listed_bodies: vec![ListEntry::new_main_body(main_body.clone())],
            main_body,
            system,
            list_state: ListState::default().with_selected(Some(0)),
            zoom_level: 1.,
            list_mapping,
            speed: DEFAULT_SPEED,
        })
    }
    pub fn run(&mut self, tui: &mut Tui) -> Result<(), Box<dyn Error>> {
        loop {
            tui.draw(|frame| self.draw_ui(frame))?;
            if event::poll(Duration::from_secs_f64(1. / FRAME_RATE))? {
                if let event::Event::Key(event) = event::read()? {
                    if event.kind == KeyEventKind::Release {
                        continue;
                    }
                    match self.current_screen {
                        AppScreen::Main => match event.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Down => self.select_next(),
                            KeyCode::Up => self.select_previous(),
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
                            KeyCode::Char('e') => self.toggle_selection_expansion()?,
                            _ => (),
                        },
                        AppScreen::Info => match event.code {
                            KeyCode::Char('q') => self.current_screen = AppScreen::Main,
                            _ => (),
                        },
                    }
                }
            }
            let mut system = self.system.borrow_mut();
            system.elapse_time(self.speed / FRAME_RATE);
            system.update_orbits();
        }
        Ok(())
    }

    pub fn selected_body_id(&self) -> BodyID {
        self.listed_bodies[self.list_state.selected().unwrap_or_default()]
            .id
            .clone()
    }

    fn select_next(&mut self) {
        self.list_state.select(match self.list_state.selected() {
            Some(i) if i == self.listed_bodies.len() - 1 => Some(i),
            Some(i) => Some(i + 1),
            None => Some(0),
        })
    }

    fn select_previous(&mut self) {
        self.list_state.select(match self.list_state.selected() {
            Some(0) | None => Some(0),
            Some(i) => Some(i - 1),
        })
    }
}
