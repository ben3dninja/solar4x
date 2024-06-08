use std::io::{Result, Stdout};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};

use crate::{
    bodies::{body_id::BodyID, BodySystem},
    ui::ui,
};

type Tui = Terminal<CrosstermBackend<Stdout>>;

const DEFAULT_BODY: &str = "soleil";

pub enum AppScreen {
    Main,
    Info,
}

pub struct App {
    pub current_screen: AppScreen,
    pub main_body: BodyID,
    pub bodies: BodySystem,
    pub list_state: ListState,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_screen: AppScreen::Main,
            main_body: BodyID::from(DEFAULT_BODY),
            bodies: BodySystem::simple_solar_system()?,
            list_state: ListState::default().with_selected(Some(1)),
        })
    }
    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        loop {
            tui.draw(|frame| ui(frame, self))?;
            if let event::Event::Key(event) = event::read()? {
                if event.kind == KeyEventKind::Release {
                    continue;
                }
                match self.current_screen {
                    AppScreen::Main => match event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => self.list_state.select(match self.list_state.selected() {
                            Some(i) if i == self.bodies.number() - 1 => Some(i),
                            Some(i) => Some(i + 1),
                            None => Some(0),
                        }),
                        KeyCode::Up => self.list_state.select(match self.list_state.selected() {
                            Some(0) => Some(0),
                            Some(i) => Some(i - 1),
                            None => None,
                        }),
                        _ => (),
                    },
                    AppScreen::Info => todo!(),
                }
            }
        }
        Ok(())
    }
}
